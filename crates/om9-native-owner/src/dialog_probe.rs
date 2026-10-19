// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in measurement of a modal belonging to this process and root UI thread.
//! No desktop input, arbitrary title search, or foreign-window control.
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM},
    System::Threading::{GetCurrentProcessId, GetCurrentThreadId},
    UI::Input::KeyboardAndMouse::IsWindowEnabled,
    UI::WindowsAndMessaging::{
        EnumThreadWindows, GA_ROOT, GW_OWNER, GetAncestor, GetClassNameW, GetDlgItem, GetWindow,
        GetWindowTextW, GetWindowThreadProcessId, IsWindow, IsWindowVisible, PostMessageW,
        WM_CLOSE, WM_COMMAND,
    },
};
use windows_sys::core::BOOL;

/// Explicit own-app audit only: post an OS close request to the supplied root.
/// No search, foreign process, desktop input or arbitrary message is accepted.
pub fn post_owned_root_close(root: usize) -> std::io::Result<()> {
    let mut pid = 0;
    let window = root as HWND;
    // SAFETY: Accept only the current thread/process root. No buffer or
    // callback crosses the asynchronous WM_CLOSE message.
    unsafe {
        if root == 0
            || IsWindow(window) == 0
            || GetWindowThreadProcessId(window, &mut pid) != GetCurrentThreadId()
            || pid != GetCurrentProcessId()
            || GetAncestor(window, GA_ROOT) != window
        {
            return Err(std::io::Error::other(
                "close probe requires this thread's root window",
            ));
        }
        if PostMessageW(window, WM_CLOSE, 0, 0) == 0 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum DialogKind {
    Open,
    Save,
}
impl DialogKind {
    fn title(self) -> &'static str {
        match self {
            Self::Open => "Open OpenMatrix9 document",
            Self::Save => "Save OpenMatrix9 document",
        }
    }
}
#[derive(Debug, Clone)]
pub struct DialogReceipt {
    pub pid: u32,
    pub thread: u32,
    pub observed: bool,
    pub close_posted: bool,
    pub accept_posted: bool,
    pub destroyed: bool,
    pub elapsed_ms: u128,
}
pub struct ModalCancelProbe {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<DialogReceipt>>,
}
struct Search {
    parent: usize,
    pid: u32,
    thread: u32,
    title: Vec<u16>,
    found: usize,
}

fn matches_owned(window: HWND, search: &Search) -> bool {
    let mut pid = 0;
    // SAFETY: Win32 validates HWNDs. Buffers below are bounded live arrays.
    unsafe {
        if GetWindowThreadProcessId(window, &mut pid) != search.thread
            || pid != search.pid
            || GetWindow(window, GW_OWNER) as usize != search.parent
            || IsWindowVisible(window) == 0
        {
            return false;
        }
        let mut class = [0u16; 32];
        let n = GetClassNameW(window, class.as_mut_ptr(), class.len() as i32);
        if n <= 0
            || class[..n as usize]
                != [
                    b'#' as u16,
                    b'3' as u16,
                    b'2' as u16,
                    b'7' as u16,
                    b'7' as u16,
                    b'0' as u16,
                ]
        {
            return false;
        }
        let mut title = [0u16; 96];
        let n = GetWindowTextW(window, title.as_mut_ptr(), title.len() as i32);
        n > 0 && title[..n as usize] == search.title
    }
}
unsafe extern "system" fn visit(window: HWND, data: LPARAM) -> BOOL {
    // SAFETY: EnumThreadWindows is synchronous; `data` points to the exclusive
    // Search on this caller's stack and never escapes the callback.
    let search = unsafe { &mut *(data as *mut Search) };
    if matches_owned(window, search) {
        search.found = window as usize;
        0
    } else {
        1
    }
}
impl ModalCancelProbe {
    pub fn arm(parent: usize, kind: DialogKind) -> std::io::Result<Self> {
        Self::arm_action(parent, kind, false)
    }
    pub fn arm_accept(parent: usize, kind: DialogKind) -> std::io::Result<Self> {
        Self::arm_action(parent, kind, true)
    }
    fn arm_action(parent: usize, kind: DialogKind, accept: bool) -> std::io::Result<Self> {
        let mut pid = 0;
        // SAFETY: Only query the supplied root, then require our own process.
        let ui_thread = unsafe { GetWindowThreadProcessId(parent as HWND, &mut pid) };
        if parent == 0 || ui_thread == 0 || pid != unsafe { GetCurrentProcessId() } {
            return Err(std::io::Error::other(
                "probe requires this process's root window",
            ));
        }
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let worker = thread::Builder::new()
            .name("om9-owned-dialog-cancel-probe".into())
            .spawn(move || {
                let started = Instant::now();
                let mut search = Search {
                    parent,
                    pid,
                    thread: ui_thread,
                    title: kind.title().encode_utf16().collect(),
                    found: 0,
                };
                let mut receipt = DialogReceipt {
                    pid,
                    thread: ui_thread,
                    observed: false,
                    close_posted: false,
                    accept_posted: false,
                    destroyed: false,
                    elapsed_ms: 0,
                };
                while started.elapsed() < Duration::from_secs(5)
                    && !worker_stop.load(Ordering::Acquire)
                {
                    if search.found == 0 {
                        // SAFETY: Enumerate only our verified root's UI thread; the
                        // callback additionally verifies PID, direct owner, class and title.
                        unsafe {
                            EnumThreadWindows(
                                ui_thread,
                                Some(visit),
                                &mut search as *mut Search as LPARAM,
                            );
                        }
                    }
                    if search.found != 0 && unsafe { IsWindow(search.found as HWND) } == 0 {
                        receipt.destroyed = true;
                        break;
                    }
                    if search.found != 0
                        && !receipt.close_posted
                        && !receipt.accept_posted
                        && matches_owned(search.found as HWND, &search)
                    {
                        receipt.observed = true;
                        if accept {
                            // SAFETY: Only the verified dialog's enabled IDOK
                            // child is accepted; no arbitrary command/input.
                            unsafe {
                                let button = GetDlgItem(search.found as HWND, 1);
                                let mut button_pid = 0;
                                if !button.is_null()
                                    && IsWindowEnabled(button) != 0
                                    && GetAncestor(button, GA_ROOT) == search.found as HWND
                                    && GetWindowThreadProcessId(button, &mut button_pid)
                                        == ui_thread
                                    && button_pid == pid
                                {
                                    receipt.accept_posted = PostMessageW(
                                        search.found as HWND,
                                        WM_COMMAND,
                                        1,
                                        button as LPARAM,
                                    ) != 0;
                                }
                            }
                        } else {
                            // SAFETY: Revalidated same-process/thread/owner modal.
                            receipt.close_posted =
                                unsafe { PostMessageW(search.found as HWND, WM_CLOSE, 0, 0) != 0 };
                        }
                    }
                    thread::sleep(Duration::from_millis(20));
                }
                // The blocking chooser can return and request stop just after destruction.
                if search.found != 0 {
                    receipt.destroyed = unsafe { IsWindow(search.found as HWND) } == 0;
                }
                receipt.elapsed_ms = started.elapsed().as_millis();
                receipt
            })?;
        Ok(Self {
            stop,
            thread: Some(worker),
        })
    }
    pub fn finish(mut self) -> std::io::Result<DialogReceipt> {
        self.stop.store(true, Ordering::Release);
        self.thread
            .take()
            .expect("probe worker")
            .join()
            .map_err(|_| std::io::Error::other("dialog probe panicked"))
    }
}
impl Drop for ModalCancelProbe {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.thread.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_missing_or_foreign_root_without_starting_worker() {
        assert!(ModalCancelProbe::arm(0, DialogKind::Open).is_err());
        assert!(ModalCancelProbe::arm(1, DialogKind::Save).is_err());
    }
    #[test]
    fn root_close_refuses_missing_and_foreign_handles() {
        assert!(post_owned_root_close(0).is_err());
        assert!(post_owned_root_close(1).is_err());
    }
    #[test]
    fn accept_probe_refuses_missing_owner_without_spawning_a_worker() {
        assert!(ModalCancelProbe::arm_accept(0, DialogKind::Open).is_err());
        assert!(ModalCancelProbe::arm_accept(1, DialogKind::Save).is_err());
    }
}
