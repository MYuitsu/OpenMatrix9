// SPDX-License-Identifier: LGPL-2.1-or-later
//! Held immutable private inputs; native writing/publication is separate.
use crate::WireOutputPlan;
use om9_core::{
    Error, Result,
    budget::Usage,
    identity::SourceStamp,
    transaction::{Cancellation, TransactionCoordinator},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};
fn io(error: std::io::Error) -> Error {
    Error::Io(error.to_string())
}
struct HeldInput {
    path: PathBuf,
    held: Option<File>,
    sha256: [u8; 32],
    bytes: u64,
    created: bool,
}
impl Drop for HeldInput {
    fn drop(&mut self) {
        self.held.take();
        if self.created {
            let _ = fs::remove_file(&self.path);
        }
    }
}
pub struct StagedRetainedRequest<'a> {
    wire: &'a WireOutputPlan<'a>,
    creation_cancel: Cancellation,
    dir: PathBuf,
    path: PathBuf,
    output: PathBuf,
    files: Vec<HeldInput>,
    request_sha256: [u8; 32],
    owned: Usage,
}
impl Drop for StagedRetainedRequest<'_> {
    fn drop(&mut self) {
        self.files.clear();
        let _ = fs::remove_file(&self.output);
        let _ = fs::remove_dir(&self.dir);
    }
}
impl StagedRetainedRequest<'_> {
    pub(crate) fn wire(&self) -> &WireOutputPlan<'_> {
        self.wire
    }
    pub fn request_path(&self) -> &Path {
        &self.path
    }
    pub fn output_path(&self) -> &Path {
        &self.output
    }
    pub fn request_sha256(&self) -> [u8; 32] {
        self.request_sha256
    }
    pub fn owned_usage(&self) -> Usage {
        self.owned
    }
    /// Child execution polls original, staging and caller cancellation together.
    /// An unrepresentable union refuses before starting a native process.
    pub fn native_cancellation(&self, cancel: &Cancellation) -> Result<Cancellation> {
        self.check_cancel(cancel)?;
        self.wire
            .inputs()
            .native_cancellation(&self.creation_cancel)?
            .either(cancel)
    }
    fn check_cancel(&self, cancel: &Cancellation) -> Result<()> {
        if self.creation_cancel.is_canceled() {
            return Err(Error::Canceled);
        }
        self.wire.inputs().check_cancel(cancel)
    }
    pub fn revalidate(&self, c: &TransactionCoordinator, cancel: &Cancellation) -> Result<()> {
        self.check_cancel(cancel)?;
        self.wire.revalidate(c, cancel)?;
        for file in &self.files {
            self.check_cancel(cancel)?;
            let held = file
                .held
                .as_ref()
                .ok_or(Error::Missing("held retained input"))?;
            let (sha256, bytes) = self.fingerprint(held, cancel)?;
            if (sha256, bytes) != (file.sha256, file.bytes) {
                return Err(Error::Invalid("retained stage bytes changed"));
            }
        }
        self.check_cancel(cancel)?;
        self.wire.revalidate(c, cancel)
    }
    fn fingerprint(&self, file: &File, cancel: &Cancellation) -> Result<([u8; 32], u64)> {
        let expected = file.metadata().map_err(io)?.len();
        let mut reader = file.try_clone().map_err(io)?;
        reader.seek(SeekFrom::Start(0)).map_err(io)?;
        let mut hash = Sha256::new();
        let mut buf = [0; 8192];
        let mut bytes = 0u64;
        loop {
            self.check_cancel(cancel)?;
            let n = reader.read(&mut buf).map_err(io)?;
            if n == 0 {
                break;
            }
            bytes = bytes
                .checked_add(n as u64)
                .ok_or(Error::Overflow("retained input bytes"))?;
            if bytes > expected {
                return Err(Error::Invalid("growing retained input"));
            }
            hash.update(&buf[..n]);
        }
        if bytes != expected {
            return Err(Error::Invalid("retained input length changed"));
        }
        Ok((hash.finalize().into(), bytes))
    }
    fn materialize(
        &mut self,
        name: &str,
        bytes: &[u8],
        expected: [u8; 32],
        cancel: &Cancellation,
    ) -> Result<()> {
        self.check_cancel(cancel)?;
        let path = self.dir.join(name);
        self.files.push(HeldInput {
            path,
            held: None,
            sha256: expected,
            bytes: bytes.len() as u64,
            created: false,
        });
        let path = &self
            .files
            .last()
            .ok_or(Error::Missing("retained source slot"))?
            .path;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(io)?;
        self.files
            .last_mut()
            .ok_or(Error::Missing("created retained input slot"))?
            .created = true;
        let path = &self
            .files
            .last()
            .ok_or(Error::Missing("retained source slot"))?
            .path;
        for block in bytes.chunks(8192) {
            self.check_cancel(cancel)?;
            file.write_all(block).map_err(io)?;
        }
        file.flush().map_err(io)?;
        file.sync_all().map_err(io)?;
        drop(file);
        use std::os::windows::fs::OpenOptionsExt;
        let held = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(path)
            .map_err(io)?;
        let (sha256, len) = self.fingerprint(&held, cancel)?;
        if sha256 != expected || len != bytes.len() as u64 {
            return Err(Error::Invalid("retained source stage correspondence"));
        }
        self.files
            .last_mut()
            .ok_or(Error::Missing("retained source stage slot"))?
            .held = Some(held);
        Ok(())
    }
}
impl WireOutputPlan<'_> {
    pub fn stage_retained_request<'a>(
        &'a self,
        c: &TransactionCoordinator,
        parent: &Path,
        cancel: &Cancellation,
        mut external: Usage,
    ) -> Result<StagedRetainedRequest<'a>> {
        self.revalidate(c, cancel)?;
        self.inputs().native_cancellation(cancel)?;
        if parent.as_os_str().len() > 8192 {
            return Err(Error::Budget("retained parent path bytes"));
        }
        let count = self
            .inputs()
            .plan()
            .overlay
            .namespaces
            .len()
            .checked_add(1)
            .ok_or(Error::Overflow("retained stage input count"))?;
        let limits = c.limits();
        let mut owned = Usage::array::<StagedRetainedRequest>(1)?;
        owned.add(Usage::array::<HeldInput>(count)?, limits)?;
        owned.add(Usage::array::<usize>(12)?, limits)?;
        owned.add(
            Usage {
                bytes: (count as u64)
                    .checked_mul(128 * 1024)
                    .ok_or(Error::Overflow("retained staging paths"))?,
                ..Usage::default()
            },
            limits,
        )?;
        external.add(owned, limits)?;
        let mut held = external;
        held.add(self.inputs().owned_usage(), limits)?;
        held.add(self.owned_usage(), limits)?;
        held.add(
            Usage::array::<SourceStamp>(self.inputs().capture().sources.capacity())?,
            limits,
        )?;
        c.retained_usage_with_held_usage(held)?;
        let parent = parent.canonicalize().map_err(io)?;
        if parent.as_os_str().len() > 8192 || !parent.is_dir() {
            return Err(Error::Invalid("retained staging parent"));
        }
        let packet = self.encode_retained_request(c, cancel, external)?;
        let request_sha256 = Sha256::digest(&packet).into();
        let mut files = Vec::new();
        files
            .try_reserve_exact(count)
            .map_err(|_| Error::Budget("retained input file slots"))?;
        self.inputs().check_cancel(cancel)?;
        let dir = parent.join(format!(".om9-retained-current-{}", Uuid::new_v4()));
        fs::create_dir(&dir).map_err(io)?;
        let mut stage = StagedRetainedRequest {
            wire: self,
            creation_cancel: cancel.clone(),
            path: dir.join("request.json"),
            output: dir.join("current.3dm"),
            dir,
            files,
            request_sha256,
            owned,
        };
        for ns in &self.inputs().plan().overlay.namespaces {
            stage.check_cancel(cancel)?;
            let resource = self
                .inputs()
                .capture()
                .document()
                .resources
                .get(&ns.source.resource)
                .ok_or(Error::Missing("retained stage resource"))?;
            if resource.sha256() != ns.source.sha256 {
                return Err(Error::Invalid("retained stage source resource identity"));
            }
            stage.materialize(
                &format!("{}.3dm", ns.namespace.id.0),
                resource.payload()?,
                ns.source.sha256,
                cancel,
            )?;
        }
        stage.materialize("request.json", &packet, request_sha256, cancel)?;
        drop(packet);
        stage.revalidate(c, cancel)?;
        Ok(stage)
    }
}
use uuid::Uuid;
