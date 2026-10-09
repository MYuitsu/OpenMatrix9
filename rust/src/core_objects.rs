// SPDX-License-Identifier: LGPL-2.1-or-later
//! Session identity bookkeeping. No native addresses or document mutations.
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Mutex, OnceLock};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stamp {
    pub document: u64,
    pub object: u64,
    pub generation: u64,
    pub revision: u64,
    pub epoch: u64,
}

struct Document {
    epoch: u64,
    objects: HashMap<u64, Stamp>,
}

pub struct Registry {
    documents: HashMap<u64, Document>,
    next_document: Option<u64>,
    next_generation: Option<u64>,
    limit: u64,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self::with_counter_limit(u64::MAX)
    }

    /// A bounded counter domain, useful for exercising exhaustion without wrap.
    pub fn with_counter_limit(limit: u64) -> Self {
        Self {
            documents: HashMap::new(),
            next_document: (limit > 0).then_some(1),
            next_generation: (limit > 0).then_some(1),
            limit,
        }
    }

    fn allocate(counter: &mut Option<u64>, limit: u64) -> Option<u64> {
        let value = (*counter)?;
        *counter = value.checked_add(1).filter(|next| *next <= limit);
        Some(value)
    }

    pub fn document_open(&mut self) -> Option<u64> {
        let id = Self::allocate(&mut self.next_document, self.limit)?;
        self.documents.insert(
            id,
            Document {
                epoch: 1,
                objects: HashMap::new(),
            },
        );
        Some(id)
    }

    pub fn document_close(&mut self, document: u64) {
        self.documents.remove(&document);
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn document_invalidate(&mut self, document: u64) -> bool {
        let Some(doc) = self.documents.get_mut(&document) else {
            return false;
        };
        let Some(epoch) = doc.epoch.checked_add(1).filter(|v| *v <= self.limit) else {
            self.documents.remove(&document);
            return false;
        };
        doc.epoch = epoch;
        true
    }

    pub fn observe(&mut self, document: u64, object: u64) -> Option<Stamp> {
        if object == 0 {
            return None;
        }
        let doc = self.documents.get_mut(&document)?;
        let stamp = match doc.objects.get(&object) {
            Some(stamp) => Stamp {
                epoch: doc.epoch,
                ..*stamp
            },
            None => Stamp {
                document,
                object,
                generation: Self::allocate(&mut self.next_generation, self.limit)?,
                revision: 1,
                epoch: doc.epoch,
            },
        };
        doc.objects.insert(object, stamp);
        Some(stamp)
    }

    pub fn changed(&mut self, document: u64, object: u64) -> bool {
        let Some(doc) = self.documents.get_mut(&document) else {
            return false;
        };
        let Some(stamp) = doc.objects.get_mut(&object) else {
            return false;
        };
        let Some(revision) = stamp.revision.checked_add(1).filter(|v| *v <= self.limit) else {
            doc.objects.remove(&object);
            return false;
        };
        stamp.revision = revision;
        true
    }

    pub fn removed(&mut self, document: u64, object: u64) {
        if let Some(doc) = self.documents.get_mut(&document) {
            doc.objects.remove(&object);
        }
    }

    pub fn validate(&self, stamp: Stamp) -> bool {
        self.documents.get(&stamp.document).is_some_and(|doc| {
            stamp.epoch == doc.epoch && doc.objects.get(&stamp.object) == Some(&stamp)
        })
    }
}

fn global<T: Default>(f: impl FnOnce(&mut Registry) -> T) -> T {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    catch_unwind(AssertUnwindSafe(|| {
        REGISTRY
            .get_or_init(|| Mutex::new(Registry::new()))
            .lock()
            .map(|mut registry| f(&mut registry))
            .unwrap_or_default()
    }))
    .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_object_document_open() -> u64 {
    global(|r| r.document_open().unwrap_or(0))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_object_document_close(document: u64) {
    global(|r| r.document_close(document));
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_object_document_invalidate(document: u64) -> bool {
    global(|r| r.document_invalidate(document))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_object_changed(document: u64, object: u64) -> bool {
    global(|r| r.changed(document, object))
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_object_removed(document: u64, object: u64) {
    global(|r| r.removed(document, object));
}

/// # Safety
/// A nonnull `out` must be aligned, writable for one Stamp, and exclusively
/// accessible for this call. It is never retained; failure leaves it unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_object_observe(document: u64, object: u64, out: *mut Stamp) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        if out.is_null() {
            return false;
        }
        let Some(stamp) = global(|r| r.observe(document, object)) else {
            return false;
        };
        // SAFETY: the caller provides writable, aligned exclusive storage.
        unsafe {
            out.write(stamp);
        }
        true
    }))
    .unwrap_or(false)
}

/// # Safety
/// A nonnull `stamp` must be aligned, initialized and readable for one Stamp,
/// with no concurrent writes during this call. The pointer is never retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_object_validate(stamp: *const Stamp) -> bool {
    catch_unwind(AssertUnwindSafe(|| {
        if stamp.is_null() {
            return false;
        }
        // SAFETY: the caller supplies readable initialized stable storage.
        let value = unsafe { stamp.read() };
        global(|r| r.validate(value))
    }))
    .unwrap_or(false)
}
