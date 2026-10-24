// SPDX-License-Identifier: LGPL-2.1-or-later
//! Explicit native plugin loading and host-owned, atomic document commands.
//! Native plugins run with the application's privileges; this is not a sandbox.
#![deny(unsafe_op_in_unsafe_fn)]

pub mod bundle;
pub mod package;
mod staged;
pub use om9_sdk::{ChangeEntry, ProjectChange};
pub use staged::ProjectState;

use libloading::Library;
use om9_core::{
    identity::{DocumentId, ObjectId},
    transaction::{TransactionCoordinator, TransactionRequest},
};
use om9_sdk::{
    PluginDescriptor, PluginMessage, PluginRequest, Result, SdkError,
    abi::{self, HostApi, ReplyCapture, ReplyFn},
};
use serde_json::Value;
use std::{ffi::c_void, path::Path};

pub struct PluginRun {
    pub request: Option<TransactionRequest>,
    pub result: Value,
    /// Bounded progress/warning reports staged by the command.
    pub messages: Vec<PluginMessage>,
    /// Replacement selection staged by the command; `None` keeps the current one.
    pub selection: Option<Vec<ObjectId>>,
    /// The command asked the host to save the project after it commits.
    pub save: bool,
    /// The command asked the host to start a Save As dialog after it commits.
    pub save_as: bool,
}

type Entry = unsafe extern "C" fn(*const HostApi, *const u8, usize, *mut c_void, ReplyFn) -> i32;

/// The library is dropped before its private extraction directory. There is no
/// global search path, automatic discovery, or retained callback into a command.
pub struct LoadedPlugin {
    library: Library,
    extracted: package::ExtractedLibrary,
    descriptor: PluginDescriptor,
    _thread_bound: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl LoadedPlugin {
    /// Loading explicitly selected native code executes its platform loader hooks.
    pub fn open(path: &Path) -> Result<Self> {
        let package = package::read(path)?;
        let extracted = package.extract_current()?;
        // SAFETY: native loading is the deliberate purpose of this API. The
        // package was bounded, checked for target/format, and its selected hash
        // verified before writing into a fresh private directory. Native code
        // itself must be trusted by the caller, just like any installed program.
        let library = unsafe { Library::new(&extracted.path) }.map_err(|e| error("load", e))?;
        let mut plugin = Self {
            library,
            extracted,
            descriptor: package.manifest.plugin,
            _thread_bound: std::marker::PhantomData,
        };
        let descriptor: PluginDescriptor =
            serde_json::from_value(plugin.invoke(std::ptr::null(), &PluginRequest::Describe)?)
                .map_err(|e| error("descriptor", e))?;
        package::validate_descriptor(&descriptor)?;
        if descriptor != plugin.descriptor {
            return Err(error(
                "descriptor",
                "manifest and native plugin descriptors differ",
            ));
        }
        plugin.descriptor = descriptor;
        Ok(plugin)
    }

    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    /// Prepare an owned request. Neither success nor failure publishes anything;
    /// the app must prepare its scene, then use its existing coordinator commit.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &self,
        coordinator: &TransactionCoordinator,
        document: DocumentId,
        selection: &[ObjectId],
        project: staged::ProjectState,
        last_change: Option<ProjectChange>,
        command: &str,
        arguments: Value,
    ) -> Result<PluginRun> {
        if !self.descriptor.commands.iter().any(|c| c.id == command) {
            return Err(error("command", "command is not declared by this plugin"));
        }
        let (_, world) = coordinator.capture_context(None).map_err(core_error)?;
        let document = world
            .documents
            .get(&document)
            .ok_or_else(|| error("document", "document is not open"))?;
        let mut staged =
            staged::Staged::new(coordinator, document, selection, coordinator.limits())?;
        staged.set_project(project);
        staged.set_last_change(last_change);
        let host = HostApi {
            abi_version: abi::ABI_VERSION,
            struct_size: std::mem::size_of::<HostApi>() as u32,
            context: (&mut staged as *mut staged::Staged<'_>).cast(),
            call: Some(staged::call),
        };
        let result = self.invoke(
            &host,
            &PluginRequest::Execute {
                command: command.into(),
                arguments,
            },
        )?;
        staged.finish(format!("{} / {command}", self.descriptor.name), result)
    }

    fn invoke(&self, host: *const HostApi, request: &PluginRequest) -> Result<Value> {
        let input = serde_json::to_vec(request).map_err(|e| error("json", e))?;
        if input.len() > abi::MAX_MESSAGE_BYTES {
            return Err(error("budget", "plugin request exceeds byte limit"));
        }
        let mut output = ReplyCapture::default();
        // SAFETY: symbol name versions the ABI. Both buffers and callback state
        // stay alive for the synchronous invocation; library outlives symbol.
        // A malicious native library cannot be made safe by an ABI signature.
        let status = unsafe {
            let entry = self
                .library
                .get::<Entry>(b"om9_plugin_entry_v1\0")
                .map_err(|e| error("entry", e))?;
            entry(
                host,
                input.as_ptr(),
                input.len(),
                (&mut output as *mut ReplyCapture).cast(),
                abi::capture_reply,
            )
        };
        let bytes = output.finish(status)?;
        serde_json::from_slice(&bytes).map_err(|e| error("json", e))
    }

    pub fn loaded_path(&self) -> &Path {
        &self.extracted.path
    }
}

pub(crate) fn error(code: &str, message: impl std::fmt::Display) -> SdkError {
    SdkError {
        code: code.into(),
        message: message.to_string(),
    }
}
pub(crate) fn core_error(error: om9_core::Error) -> SdkError {
    self::error("host", error)
}
