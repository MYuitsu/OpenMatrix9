// SPDX-License-Identifier: LGPL-2.1-or-later
//! Host adapter for the native openNURBS 3DM reader/writer.
//!
//! The native executable is admitted by path **and** SHA-256 before it is run; the path and digest
//! come from trusted application configuration, never from a document or a user data file. The Rust
//! side owns the transaction, identity, import/export capture and publication; the native side owns
//! only openNURBS parsing/writing.

use om9_core::{
    Error, Result,
    identity::DocumentId,
    transaction::{Cancellation, TransactionCoordinator},
};
use om9_exchange::{
    ExportCapture, ExportReceipt, ImportCapture, ImportMode, ImportOptions, ImportReceipt,
    NativeExchange,
};
use om9_native_owner::{NativeProgram, ProcessLimits};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

pub const NATIVE_PATH_ENV: &str = "OM9_EXCHANGE_NATIVE";
pub const NATIVE_SHA256_ENV: &str = "OM9_EXCHANGE_NATIVE_SHA256";

/// A configured native 3DM reader/writer and its process limits.
pub struct ThreeDmHost {
    exchange: NativeExchange,
    program: PathBuf,
    fingerprint: [u8; 32],
}

fn parse_fingerprint(text: &str) -> Result<[u8; 32]> {
    let text = text.trim();
    if text.len() != 64 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::Invalid("native exchange fingerprint"));
    }
    let mut fingerprint = [0u8; 32];
    for (index, slot) in fingerprint.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| Error::Invalid("native exchange fingerprint"))?;
    }
    Ok(fingerprint)
}

impl ThreeDmHost {
    /// Admit `path` after checking the trusted fingerprint, then build the exchange owner.
    pub fn admit(path: &Path, fingerprint: [u8; 32]) -> Result<Self> {
        if !path.is_absolute() {
            return Err(Error::Invalid("native exchange absolute path"));
        }
        let program = NativeProgram::admit(path, fingerprint)
            .map_err(|error| Error::Io(format!("native exchange admission: {error:?}")))?;
        let exchange = NativeExchange::new(program, ProcessLimits::default())?;
        Ok(Self {
            exchange,
            program: path.to_path_buf(),
            fingerprint,
        })
    }

    /// Development/CLI configuration: `OM9_EXCHANGE_NATIVE` (absolute path) and
    /// `OM9_EXCHANGE_NATIVE_SHA256` (64 hex digits). A packaged application should provide the same
    /// values from its signed layout/manifest instead of the environment.
    pub fn from_environment() -> Result<Self> {
        let path =
            std::env::var_os(NATIVE_PATH_ENV).ok_or(Error::Missing("OM9_EXCHANGE_NATIVE"))?;
        let text = std::env::var(NATIVE_SHA256_ENV)
            .map_err(|_| Error::Missing("OM9_EXCHANGE_NATIVE_SHA256"))?;
        Self::admit(Path::new(&path), parse_fingerprint(&text)?)
    }

    /// Packaged layout: `om9-exchange-native.exe` plus its `.sha256` sibling next to the
    /// application executable (or in `bin/`). The digest file is part of the signed layout; this
    /// function never trusts a digest selected by a document.
    pub fn from_application_layout() -> Result<Self> {
        let executable = std::env::current_exe().map_err(|error| Error::Io(error.to_string()))?;
        let directory = executable
            .parent()
            .ok_or(Error::Missing("application directory"))?;
        Self::from_directory(directory)
    }

    /// Load the layout from an explicit application directory. Exposed for packaging tests and
    /// launchers whose executable directory is not the layout root.
    pub fn from_directory(directory: &Path) -> Result<Self> {
        for candidate in [directory.to_path_buf(), directory.join("bin")] {
            let program = candidate.join("om9-exchange-native.exe");
            if !program.is_file() {
                continue;
            }
            let digest = program.with_extension("sha256");
            let text =
                std::fs::read_to_string(&digest).map_err(|error| Error::Io(error.to_string()))?;
            return Self::admit(&program, parse_fingerprint(&text)?);
        }
        Err(Error::Missing("om9-exchange-native.exe layout"))
    }

    /// Packaged layout first, environment seam second. A deployment that forbids an environment
    /// override calls [`Self::from_application_layout`] directly.
    pub fn configured() -> Result<Self> {
        Self::from_application_layout().or_else(|_| Self::from_environment())
    }

    pub fn program(&self) -> &Path {
        &self.program
    }

    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }

    /// Import `source` into an existing document and commit it as one transaction.
    pub fn import(
        &self,
        coordinator: &mut TransactionCoordinator,
        document: DocumentId,
        source: &Path,
        cancel: &Cancellation,
    ) -> Result<ImportReceipt> {
        self.import_with_options(
            coordinator,
            document,
            source,
            &ImportOptions {
                // The product requirement is editable geometry: no retained history/source
                // tables are needed to continue editing curves, lines and meshes.
                mode: ImportMode::Geometry,
                ..ImportOptions::default()
            },
            cancel,
        )
    }

    pub fn import_with_options(
        &self,
        coordinator: &mut TransactionCoordinator,
        document: DocumentId,
        source: &Path,
        options: &ImportOptions,
        cancel: &Cancellation,
    ) -> Result<ImportReceipt> {
        let capture = ImportCapture::target(coordinator, document)?;
        let staging = std::env::temp_dir();
        let prepared = self.exchange.prepare_import(
            coordinator,
            capture,
            source,
            &staging,
            options,
            cancel,
        )?;
        let (transaction, receipt) = prepared.prepare(coordinator, cancel)?;
        coordinator.commit(transaction)?;
        Ok(receipt)
    }

    /// Export the document's current state to `target`. The native writer publishes to a private
    /// staged file first; `publish` performs the final atomic rename only after inspection.
    pub fn export(
        &self,
        coordinator: &TransactionCoordinator,
        document: DocumentId,
        target: &Path,
        cancel: &Cancellation,
    ) -> Result<ExportReceipt> {
        let capture = ExportCapture::all(coordinator, document)?;
        let validated = self
            .exchange
            .prepare(coordinator, capture, target, cancel)?;
        validated.publish(coordinator, cancel)
    }
}

/// Format a fingerprint as lowercase hex for receipts/configuration.
pub fn fingerprint_hex(fingerprint: [u8; 32]) -> String {
    fingerprint
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Native program environment key and digest used by the CLI tests and development launcher.
pub fn native_arguments(path: &Path, fingerprint: [u8; 32]) -> Vec<OsString> {
    vec![
        OsString::from(NATIVE_PATH_ENV),
        path.as_os_str().to_owned(),
        OsString::from(NATIVE_SHA256_ENV),
        OsString::from(fingerprint_hex(fingerprint)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_requires_exactly_64_hex_digits() {
        let valid = "9bea4c8066ef4a1c206b2be5a36302f8926f7fdc6087af5d20b417d0cf103ea6";
        let parsed = parse_fingerprint(valid).unwrap();
        assert_eq!(fingerprint_hex(parsed), valid);
        for invalid in [
            "",
            "00",
            &valid[..63],
            &format!("{valid}0"),
            "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
        ] {
            assert!(parse_fingerprint(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn absolute_path_and_trusted_digest_are_required() {
        let digest = [0u8; 32];
        assert!(ThreeDmHost::admit(Path::new("relative-native.exe"), digest).is_err());
        let missing = std::env::temp_dir().join("om9-no-such-native.exe");
        assert!(ThreeDmHost::admit(&missing, digest).is_err());
    }

    #[test]
    fn packaged_layout_reads_the_sibling_digest_before_admission() {
        use sha2::{Digest, Sha256};
        let folder = std::env::temp_dir().join(format!("om9-layout-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&folder).unwrap();
        let executable = folder.join("om9-exchange-native.exe");
        std::fs::write(&executable, b"owned native executable bytes").unwrap();
        let digest: [u8; 32] = Sha256::digest(std::fs::read(&executable).unwrap()).into();
        std::fs::write(
            folder.join("om9-exchange-native.sha256"),
            fingerprint_hex(digest),
        )
        .unwrap();
        let host = ThreeDmHost::from_directory(&folder).unwrap();
        assert_eq!(host.fingerprint(), digest);
        assert_eq!(host.program(), executable.as_path());
        std::fs::write(folder.join("om9-exchange-native.sha256"), "00").unwrap();
        assert!(ThreeDmHost::from_directory(&folder).is_err());
        drop(host);
        std::fs::remove_file(executable).unwrap();
        std::fs::remove_file(folder.join("om9-exchange-native.sha256")).unwrap();
        std::fs::remove_dir(folder).unwrap();
    }
}
