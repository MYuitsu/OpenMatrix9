//! Owned, document/view-scoped construction planes. Native adapters own camera,
//! grid/display state and persistence; plane edits never imply camera edits.
use crate::coordinates::Frame;
use std::collections::BTreeMap;
use std::ffi::{CStr, c_char};
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub frame: Frame,
    pub revision: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NamedPlane {
    pub id: u64,
    pub name: String,
    pub frame: Frame,
}

struct ViewHistory {
    frames: Vec<Frame>,
    cursor: usize,
    revision: u64,
}

impl ViewHistory {
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            frame: self.frames[self.cursor],
            revision: self.revision,
        }
    }
}

/// This bounded history is an OM9 storage policy, not a recovered Rhino limit.
const MAX_HISTORY: usize = 256;
/// OM9 document storage budgets, not recovered Rhino limits. The aggregate
/// UTF-8 label budget leaves room for 4096 frames/identities and JSON escaping
/// inside the native adapter's 4 MiB persisted metadata limit.
pub const MAX_NAMED_PLANES: usize = 4096;
pub const MAX_NAMED_LABEL_BYTES: usize = 512 * 1024;

#[derive(Default)]
pub struct Registry {
    views: BTreeMap<(u64, u64), ViewHistory>,
    revision: u64,
    named: BTreeMap<(u64, u64), NamedPlane>,
    named_id: u64,
}

impl Registry {
    /// Explicit world-axis choices: XY, XZ and YZ. View cameras are independent.
    pub fn set_world(&mut self, document: u64, view: u64, plane: u32) -> Result<Snapshot, String> {
        let axes = match plane {
            0 => [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            1 => [[1., 0., 0.], [0., 0., 1.], [0., -1., 0.]],
            2 => [[0., 1., 0.], [0., 0., 1.], [1., 0., 0.]],
            _ => return Err("Unknown world construction plane".into()),
        };
        self.set(document, view, Frame::new([0.; 3], axes)?)
    }
    pub fn history_available(&self, document: u64, view: u64, forward: bool) -> bool {
        self.views.get(&(document, view)).is_some_and(|history| {
            if forward {
                history.cursor + 1 < history.frames.len()
            } else {
                history.cursor > 0
            }
        })
    }
    /// Names are exact, nonempty labels. Duplicate names require an explicit
    /// rename or replacement by the caller; saving never overwrites implicitly.
    pub fn save_named(&mut self, document: u64, name: &str, frame: Frame) -> Result<u64, String> {
        Self::check_name(document, name)?;
        if self
            .named
            .keys()
            .filter(|(owner, _)| *owner == document)
            .count()
            >= MAX_NAMED_PLANES
        {
            return Err("Named construction plane table is full".into());
        }
        if self
            .named
            .iter()
            .any(|((owner, _), plane)| *owner == document && plane.name == name)
        {
            return Err("A named construction plane already has this name".into());
        }
        Self::check_name_budget(
            self.named_label_bytes(document, None)
                .saturating_add(name.len()),
        )?;
        let id = self
            .named_id
            .checked_add(1)
            .ok_or("Named construction plane identity exhausted")?;
        self.named.insert(
            (document, id),
            NamedPlane {
                id,
                name: name.into(),
                frame,
            },
        );
        self.named_id = id;
        Ok(id)
    }
    pub fn rename_named(&mut self, document: u64, id: u64, name: &str) -> Result<(), String> {
        Self::check_name(document, name)?;
        if self
            .named
            .iter()
            .any(|((owner, key), plane)| *owner == document && *key != id && plane.name == name)
        {
            return Err("A named construction plane already has this name".into());
        }
        Self::check_name_budget(
            self.named_label_bytes(document, Some(id))
                .saturating_add(name.len()),
        )?;
        self.named
            .get_mut(&(document, id))
            .ok_or("Unknown named construction plane")?
            .name = name.into();
        Ok(())
    }
    pub fn named(&self, document: u64) -> Vec<NamedPlane> {
        self.named
            .iter()
            .filter(|((owner, _), _)| *owner == document)
            .map(|(_, plane)| plane.clone())
            .collect()
    }
    pub fn restore_named(&mut self, document: u64, view: u64, id: u64) -> Result<Snapshot, String> {
        let frame = self
            .named
            .get(&(document, id))
            .ok_or("Unknown named construction plane")?
            .frame;
        self.set(document, view, frame)
    }
    /// Restore a complete document table after validating every record. Frames
    /// are already validated values; native deserializers must use Frame::new.
    pub fn import_named(&mut self, document: u64, records: &[NamedPlane]) -> Result<(), String> {
        if document == 0 || records.len() > MAX_NAMED_PLANES {
            return Err(
                "Named construction plane needs a document identity and at most 4096 records"
                    .into(),
            );
        }
        let mut incoming = BTreeMap::new();
        let mut names = std::collections::BTreeSet::new();
        let mut max_id = self.named_id;
        let mut label_bytes: usize = 0;
        for plane in records {
            Self::check_name(document, &plane.name)?;
            label_bytes = label_bytes.saturating_add(plane.name.len());
            Self::check_name_budget(label_bytes)?;
            if plane.id == 0
                || incoming
                    .insert((document, plane.id), plane.clone())
                    .is_some()
                || !names.insert(&plane.name)
            {
                return Err("Duplicate or invalid named construction plane identity/name".into());
            }
            max_id = max_id.max(plane.id);
        }
        self.named.retain(|(owner, _), _| *owner != document);
        self.named.extend(incoming);
        self.named_id = max_id;
        Ok(())
    }
    fn named_label_bytes(&self, document: u64, except: Option<u64>) -> usize {
        self.named
            .iter()
            .filter(|((owner, id), _)| *owner == document && Some(*id) != except)
            .fold(0, |total, (_, plane)| {
                total.saturating_add(plane.name.len())
            })
    }
    fn check_name_budget(bytes: usize) -> Result<(), String> {
        if bytes > MAX_NAMED_LABEL_BYTES {
            Err(
                "Named construction plane labels exceed the OM9 document limit of 512 KiB UTF-8"
                    .into(),
            )
        } else {
            Ok(())
        }
    }
    fn check_name(document: u64, name: &str) -> Result<(), String> {
        if document == 0
            || name.trim().is_empty()
            || name.len() >= 1024 * 1024
            || name.chars().any(char::is_control)
        {
            Err("Named construction plane needs a document and nonempty name".into())
        } else {
            Ok(())
        }
    }
    fn advance_revision(&mut self) -> Result<u64, String> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or("Construction plane revision exhausted")?;
        Ok(self.revision)
    }

    pub fn ensure(&mut self, document: u64, view: u64, frame: Frame) -> Result<Snapshot, String> {
        if document == 0 || view == 0 {
            return Err("Construction plane needs document and view identity".into());
        }
        if let Some(snapshot) = self.read(document, view) {
            return Ok(snapshot);
        }
        let revision = self.advance_revision()?;
        self.views.insert(
            (document, view),
            ViewHistory {
                frames: vec![frame],
                cursor: 0,
                revision,
            },
        );
        Ok(Snapshot { frame, revision })
    }

    pub fn set(&mut self, document: u64, view: u64, frame: Frame) -> Result<Snapshot, String> {
        let current = self
            .read(document, view)
            .ok_or("Unknown construction plane view")?;
        if current.frame == frame {
            return Ok(current);
        }
        let revision = self.advance_revision()?;
        let history = self
            .views
            .get_mut(&(document, view))
            .ok_or("Unknown construction plane view")?;
        history.frames.truncate(history.cursor + 1);
        if history.frames.len() == MAX_HISTORY {
            history.frames.remove(0);
        }
        history.frames.push(frame);
        history.cursor = history.frames.len() - 1;
        history.revision = revision;
        Ok(history.snapshot())
    }

    pub fn read(&self, document: u64, view: u64) -> Option<Snapshot> {
        self.views.get(&(document, view)).map(ViewHistory::snapshot)
    }

    pub fn previous(&mut self, document: u64, view: u64) -> Result<Snapshot, String> {
        self.navigate(document, view, false)
    }

    pub fn next(&mut self, document: u64, view: u64) -> Result<Snapshot, String> {
        self.navigate(document, view, true)
    }

    fn navigate(&mut self, document: u64, view: u64, forward: bool) -> Result<Snapshot, String> {
        let history = self
            .views
            .get(&(document, view))
            .ok_or("Unknown construction plane view")?;
        let cursor = if forward {
            if history.cursor + 1 == history.frames.len() {
                return Err("No next construction plane".into());
            }
            history.cursor + 1
        } else {
            history
                .cursor
                .checked_sub(1)
                .ok_or("No previous construction plane")?
        };
        let revision = self.advance_revision()?;
        let history = self
            .views
            .get_mut(&(document, view))
            .ok_or("Unknown construction plane view")?;
        history.cursor = cursor;
        history.revision = revision;
        Ok(history.snapshot())
    }

    pub fn drop_view(&mut self, document: u64, view: u64) {
        self.views.remove(&(document, view));
    }

    pub fn drop_document(&mut self, document: u64) {
        self.views.retain(|(owner, _), _| *owner != document);
        self.named.retain(|(owner, _), _| *owner != document);
    }
}

fn state() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}

/// # Safety
/// A nonnull `frame` points to 12 readable doubles: origin, X, Y, Z. No pointer
/// is retained. A null/invalid frame is rejected without changing the registry.
unsafe fn read_frame(frame: *const f64) -> Result<Frame, String> {
    if frame.is_null() {
        return Err("Missing construction plane".into());
    }
    let values = unsafe { std::slice::from_raw_parts(frame, 12) };
    let triple = |i| [values[i], values[i + 1], values[i + 2]];
    Frame::new(triple(0), [triple(3), triple(6), triple(9)])
}

/// # Safety
/// `frame` is null or points to 12 readable doubles for this call only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_ensure(document: u64, view: u64, frame: *const f64) -> bool {
    let Ok(frame) = (unsafe { read_frame(frame) }) else {
        return false;
    };
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.ensure(document, view, frame).is_ok())
}

/// # Safety
/// `frame` is null or points to 12 readable doubles for this call only.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_set(document: u64, view: u64, frame: *const f64) -> bool {
    let Ok(frame) = (unsafe { read_frame(frame) }) else {
        return false;
    };
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.set(document, view, frame).is_ok())
}

/// # Safety
/// `frame` is null or points to 12 writable doubles. `revision` is null or one
/// writable u64. Writable buffers do not alias and no pointer is retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_read(
    document: u64,
    view: u64,
    frame: *mut f64,
    revision: *mut u64,
) -> bool {
    if frame.is_null() {
        return false;
    }
    let Some(snapshot) = state()
        .lock()
        .ok()
        .and_then(|state| state.read(document, view))
    else {
        return false;
    };
    let axes = snapshot.frame.axes();
    let values = [snapshot.frame.origin(), axes[0], axes[1], axes[2]];
    unsafe {
        for (i, triple) in values.iter().enumerate() {
            std::ptr::copy_nonoverlapping(triple.as_ptr(), frame.add(3 * i), 3);
        }
        if !revision.is_null() {
            *revision = snapshot.revision;
        }
    }
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_previous(document: u64, view: u64) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.previous(document, view).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_next(document: u64, view: u64) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.next(document, view).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_set_world(document: u64, view: u64, plane: u32) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.set_world(document, view, plane).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_history_available(document: u64, view: u64, forward: bool) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|state| state.history_available(document, view, forward))
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_drop_view(document: u64, view: u64) {
    if let Ok(mut state) = state().lock() {
        state.drop_view(document, view);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_drop_document(document: u64) {
    if let Ok(mut state) = state().lock() {
        state.drop_document(document);
    }
}

/// # Safety
/// `name` is null or a readable NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_named_save(
    document: u64,
    view: u64,
    name: *const c_char,
) -> u64 {
    if name.is_null() {
        return 0;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return 0;
    };
    let Ok(mut state) = state().lock() else {
        return 0;
    };
    let Some(snapshot) = state.read(document, view) else {
        return 0;
    };
    state
        .save_named(document, name, snapshot.frame)
        .unwrap_or(0)
}

/// # Safety
/// `name` is null or a readable NUL-terminated UTF-8 string for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_named_rename(
    document: u64,
    id: u64,
    name: *const c_char,
) -> bool {
    if name.is_null() {
        return false;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return false;
    };
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.rename_named(document, id, name).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_named_restore(document: u64, view: u64, id: u64) -> bool {
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.restore_named(document, view, id).is_ok())
}

#[unsafe(no_mangle)]
pub extern "C" fn om9_cplane_named_count(document: u64) -> usize {
    state()
        .lock()
        .ok()
        .map_or(0, |state| state.named(document).len())
}

/// # Safety
/// `frame` is null or 12 writable doubles. `id` is null or a writable u64.
/// `name` is null or `capacity` writable bytes, separate from other outputs.
/// Returns required UTF-8 name bytes including NUL, or 0 for no record. A null
/// name queries size; an undersized name buffer is not written.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_named_read(
    document: u64,
    index: usize,
    id: *mut u64,
    frame: *mut f64,
    name: *mut c_char,
    capacity: usize,
) -> usize {
    let Ok(state) = state().lock() else {
        return 0;
    };
    let records = state.named(document);
    let Some(record) = records.get(index) else {
        return 0;
    };
    let required = record.name.len() + 1;
    unsafe {
        if !id.is_null() {
            *id = record.id;
        }
        if !frame.is_null() {
            let axes = record.frame.axes();
            for (i, triple) in [record.frame.origin(), axes[0], axes[1], axes[2]]
                .iter()
                .enumerate()
            {
                std::ptr::copy_nonoverlapping(triple.as_ptr(), frame.add(i * 3), 3);
            }
        }
        if !name.is_null() && capacity >= required {
            std::ptr::copy_nonoverlapping(record.name.as_ptr(), name.cast(), record.name.len());
            *name.add(record.name.len()) = 0;
        }
    }
    required
}

/// # Safety
/// For positive count, ids/names/frames contain count u64s, count readable
/// NUL-terminated UTF-8 string pointers, and count*12 readable doubles. Null is
/// permitted only when count is zero. No pointer is retained; import is atomic.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_named_import(
    document: u64,
    ids: *const u64,
    names: *const *const c_char,
    frames: *const f64,
    count: usize,
) -> bool {
    if count > 4096 || (count != 0 && (ids.is_null() || names.is_null() || frames.is_null())) {
        return false;
    }
    let mut records = Vec::with_capacity(count);
    for i in 0..count {
        let name = unsafe { *names.add(i) };
        if name.is_null() {
            return false;
        }
        let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
            return false;
        };
        let Ok(frame) = (unsafe { read_frame(frames.add(i * 12)) }) else {
            return false;
        };
        records.push(NamedPlane {
            id: unsafe { *ids.add(i) },
            name: name.into(),
            frame,
        });
    }
    state()
        .lock()
        .ok()
        .is_some_and(|mut state| state.import_named(document, &records).is_ok())
}

/// # Safety
/// points is null or 9 readable doubles; frame is null or 12 writable doubles.
/// Input/output do not alias. Validation failure leaves the output unchanged.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_cplane_three_points(points: *const f64, frame: *mut f64) -> bool {
    if points.is_null() || frame.is_null() {
        return false;
    }
    let values = unsafe { std::slice::from_raw_parts(points, 9) };
    let triple = |i| [values[i], values[i + 1], values[i + 2]];
    let Ok(plane) = Frame::from_three_points(triple(0), triple(3), triple(6)) else {
        return false;
    };
    let axes = plane.axes();
    for (i, triple) in [plane.origin(), axes[0], axes[1], axes[2]]
        .iter()
        .enumerate()
    {
        unsafe {
            std::ptr::copy_nonoverlapping(triple.as_ptr(), frame.add(i * 3), 3);
        }
    }
    true
}
