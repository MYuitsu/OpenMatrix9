// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_core::{
    identity::{DocumentId, SessionId},
    transaction::OperationContext,
};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FileKind {
    New,
    Open,
    Save,
    SaveAs,
}
impl FileKind {
    pub const ALL: [Self; 4] = [Self::New, Self::Open, Self::Save, Self::SaveAs];
    pub fn feature_id(self) -> &'static str {
        match self {
            Self::New => "OM9-FILE-002",
            Self::Open => "OM9-FILE-003",
            Self::Save => "OM9-FILE-005",
            Self::SaveAs => "OM9-FILE-006",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Open => "Open",
            Self::Save => "Save",
            Self::SaveAs => "Save As",
        }
    }
    pub fn from_feature(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.feature_id() == id)
    }
    pub fn from_command(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| {
            kind.label().eq_ignore_ascii_case(text)
                || (*kind == Self::SaveAs && text.eq_ignore_ascii_case("SaveAs"))
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePhase {
    Unsaved,
    OpenPath,
    SavePath,
    Overwrite,
    CloseReady,
}
/// Window shutdown is an internal lifecycle intent, with no catalog feature ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileIntent {
    Command(FileKind),
    WindowClose,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedDecision {
    Save,
    Discard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOutcome {
    Pending,
    Saved,
    Replaced,
    SavedPending,
    SavedAndReplaced,
    CloseReady,
    SavedAndCloseReady,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSaveReceipt {
    pub(crate) token: FileToken,
    pub(crate) fingerprint: [u8; 32],
}
impl FileSaveReceipt {
    pub fn token(&self) -> FileToken {
        self.token
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileToken {
    pub(crate) context: OperationContext,
    pub(crate) owner: (DocumentId, SessionId, u64, u64, u64, u64),
    pub(crate) serial: u64,
}
impl FileToken {
    pub fn authority(self) -> SessionId {
        self.context.authority()
    }
    pub fn serial(self) -> u64 {
        self.serial
    }
}
pub struct FileOperation {
    pub(crate) token: FileToken,
    pub(crate) intent: FileIntent,
    pub(crate) phase: FilePhase,
    pub(crate) destination: Option<PathBuf>,
}
impl FileOperation {
    pub fn token(&self) -> FileToken {
        self.token
    }
    pub fn kind(&self) -> Option<FileKind> {
        match self.intent {
            FileIntent::Command(kind) => Some(kind),
            FileIntent::WindowClose => None,
        }
    }
    pub fn intent(&self) -> FileIntent {
        self.intent
    }
    pub fn phase(&self) -> FilePhase {
        self.phase
    }
    pub fn destination(&self) -> Option<&Path> {
        self.destination.as_deref()
    }
}
pub(crate) struct FileState {
    pub path: Option<PathBuf>,
    pub saved: Option<[u8; 32]>,
    pub cache: Cell<Option<(u64, [u8; 32])>>,
    pub diagnostics: Vec<String>,
    pub operation: Option<FileOperation>,
    pub serial: u64,
    pub accepted_save: Option<FileSaveReceipt>,
}
impl Default for FileState {
    fn default() -> Self {
        Self {
            path: None,
            saved: None,
            cache: Cell::new(None),
            diagnostics: vec![],
            operation: None,
            serial: 0,
            accepted_save: None,
        }
    }
}
pub(crate) fn fingerprint(document: &om9_core::model::Document) -> om9_core::Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    struct Sink(Sha256);
    impl std::io::Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink(Sha256::new());
    serde_json::to_writer(&mut sink, document).map_err(|e| om9_core::Error::Io(e.to_string()))?;
    Ok(sink.0.finalize().into())
}
impl FileState {
    pub fn initialized(document: &om9_core::model::Document) -> om9_core::Result<Self> {
        Ok(Self {
            saved: Some(fingerprint(document)?),
            ..Self::default()
        })
    }
    pub fn usage(
        &self,
        limits: &om9_core::budget::Limits,
    ) -> om9_core::Result<om9_core::budget::Usage> {
        use om9_core::budget::Usage;
        let mut usage = Usage::array::<String>(self.diagnostics.capacity())?;
        for diagnostic in &self.diagnostics {
            usage.add(
                Usage {
                    bytes: diagnostic.capacity() as u64,
                    ..Usage::default()
                },
                limits,
            )?;
        }
        if let Some(path) = &self.path {
            usage.add(
                Usage {
                    bytes: path.capacity() as u64,
                    ..Usage::default()
                },
                limits,
            )?;
        }
        if let Some(operation) = &self.operation {
            usage.add(Usage::array::<FileOperation>(1)?, limits)?;
            usage.add(
                Usage {
                    bytes: 1024,
                    ..Usage::default()
                },
                limits,
            )?;
            if let Some(path) = &operation.destination {
                usage.add(
                    Usage {
                        bytes: path.capacity() as u64,
                        ..Usage::default()
                    },
                    limits,
                )?;
            }
        }
        Ok(usage)
    }
}
