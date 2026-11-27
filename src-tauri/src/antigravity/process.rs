//! Finding, stopping and launching Antigravity processes.
//!
//! A switch waits for the selected desktop installation and its bundled helpers
//! to close before changing credentials.

use std::borrow::Cow;
use std::path::Path;
use std::time::{Duration, Instant};

use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use crate::antigravity::install::Install;
use crate::error::{Error, Result};
pub use crate::model::{ProcessKind, RelatedProcess};

const GRACEFUL_TIMEOUT: Duration = Duration::from_secs(8);
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(15);
const POLL: Duration = Duration::from_millis(250);

fn snapshot() -> System {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    system
}

fn classify(install: Option<&Install>, process: &Process) -> Option<ProcessKind> {
    let install = install?;
    let exe = executable_path(process)?;
    classify_executable(install, &exe)
}

fn classify_executable(install: &Install, exe: &Path) -> Option<ProcessKind> {
    let file_name = exe.file_name()?.to_string_lossy().to_ascii_lowercase();
    if matches!(file_name.as_str(), "agy" | "agy.exe") {
        return None;
    }
    let language_server = file_name.starts_with("language_server");
    #[cfg(windows)]
    if is_windows_client_image(&file_name)
        && let (Ok(exe), Ok(root), Ok(main_exe)) = (
            exe.canonicalize(),
            install.root.canonicalize(),
            install.main_exe.canonicalize(),
        )
    {
        return classify_in_root(&exe, &root, &main_exe, language_server);
    }
    classify_in_root(exe, &install.root, &install.main_exe, language_server)
}

fn classify_in_root(
    exe: &Path,
    root: &Path,
    main_exe: &Path,
    language_server: bool,
) -> Option<ProcessKind> {
    exe.starts_with(root).then(|| {
        if exe == main_exe {
            ProcessKind::App
        } else if language_server {
            ProcessKind::LanguageServer
        } else {
            ProcessKind::Helper
        }
    })
}

pub(super) fn executable_path(process: &Process) -> Option<Cow<'_, Path>> {
    #[cfg(windows)]
    if is_windows_client_image(&process.name().to_string_lossy().to_ascii_lowercase())
        && let Some(path) = windows_image_path(process.pid().as_u32())
    {
        return Some(Cow::Owned(path));
    }
    process
        .exe()
        .filter(|path| path.is_absolute())
        .map(Cow::Borrowed)
}

#[cfg(windows)]
fn is_windows_client_image(lower_name: &str) -> bool {
    lower_name == "antigravity.exe" || lower_name.starts_with("language_server")
}

#[cfg(windows)]
fn windows_image_path(pid: u32) -> Option<std::path::PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    };

    // 直接查询进程映像，避免依赖正在初始化的模块列表或 MAX_PATH 长度。
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }
    // SAFETY: OpenProcess returned an owned, non-null handle.
    let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
    let mut image = vec![0_u16; 32_768];
    let mut len = image.len() as u32;
    // SAFETY: the handle and output buffer are valid for the duration of the call.
    let ok = unsafe {
        QueryFullProcessImageNameW(handle.as_raw_handle(), 0, image.as_mut_ptr(), &mut len)
    };
    (ok != 0).then(|| OsString::from_wide(&image[..len as usize]).into())
}

fn related(system: &System, install: Option<&Install>) -> Vec<(Pid, ProcessKind)> {
    system
        .processes()
        .iter()
        .filter_map(|(pid, process)| classify(install, process).map(|kind| (*pid, kind)))
        .collect()
}

/// The Antigravity processes running right now.
pub fn scan(install: Option<&Install>) -> Vec<RelatedProcess> {
    related(&snapshot(), install)
        .into_iter()
        .map(|(pid, kind)| RelatedProcess {
            pid: pid.as_u32(),
            kind,
        })
        .collect()
}

pub fn is_app_running(install: Option<&Install>) -> bool {
    scan(install).iter().any(|p| p.kind == ProcessKind::App)
}

/// Requests desktop exit; a stuck process aborts the switch before any write.
pub async fn stop_all(install: Option<&Install>) -> Result<()> {
    let system = snapshot();
    for (pid, kind) in related(&system, install) {
        if kind == ProcessKind::App
            && let Some(process) = system.process(pid)
        {
            request_exit(process);
        }
    }
    if wait_until(GRACEFUL_TIMEOUT, || scan(install).is_empty()).await {
        Ok(())
    } else {
        Err(Error::ProcessStuck(scan(install).len()))
    }
}

/// Launches the IDE and waits until its main process is up.
pub async fn launch(install: &Install) -> Result<()> {
    spawn_app(&install.root, &install.main_exe).map_err(|err| Error::Launch(err.to_string()))?;
    if wait_until(LAUNCH_TIMEOUT, || is_app_running(Some(install))).await {
        Ok(())
    } else {
        Err(Error::Launch("Antigravity did not start in time".into()))
    }
}

async fn wait_until(timeout: Duration, mut done: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if done() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(POLL).await;
    }
}

#[cfg(unix)]
fn request_exit(process: &Process) {
    // SIGTERM lets Electron shut down; an Apple Event
    // quit could block on dialogs and needs Automation permission.
    process.kill_with(sysinfo::Signal::Term);
}

#[cfg(windows)]
fn request_exit(process: &Process) {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
    // Without /F, taskkill posts WM_CLOSE so the app can close its windows.
    let _ = std::process::Command::new("taskkill")
        .args(["/PID", &process.pid().as_u32().to_string()])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

#[cfg(target_os = "macos")]
fn spawn_app(root: &Path, _main_exe: &Path) -> std::io::Result<()> {
    let status = std::process::Command::new("/usr/bin/open")
        .arg(root)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("open exited with {status}")))
    }
}

#[cfg(windows)]
fn spawn_app(root: &Path, main_exe: &Path) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};
    std::process::Command::new(main_exe)
        .current_dir(root)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map(drop)
}

#[cfg(not(any(target_os = "macos", windows)))]
fn spawn_app(root: &Path, main_exe: &Path) -> std::io::Result<()> {
    std::process::Command::new(main_exe)
        .current_dir(root)
        .spawn()
        .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_classification_stays_inside_the_selected_installation() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Antigravity");
        let install = Install {
            main_exe: root.join("Antigravity.exe"),
            bin_dir: root.join("resources").join("bin"),
            root,
        };
        assert_eq!(
            classify_executable(&install, &install.main_exe),
            Some(ProcessKind::App),
        );
        assert_eq!(
            classify_executable(&install, &install.bin_dir.join("language_server_x64.exe")),
            Some(ProcessKind::LanguageServer),
        );
        assert_eq!(
            classify_executable(&install, &install.root.join("crashpad_handler.exe")),
            Some(ProcessKind::Helper),
        );
        assert_eq!(
            classify_executable(&install, &install.bin_dir.join("agy.exe")),
            None
        );
        assert_eq!(
            classify_executable(
                &install,
                &dir.path().join("Antigravity IDE").join("Antigravity.exe")
            ),
            None,
        );
    }

    #[cfg(windows)]
    #[test]
    fn recognizes_windows_installation_with_different_path_case() {
        let dir = tempfile::tempdir().unwrap();
        // 官方 Windows 安装目录使用小写，旧的标准候选路径以大写开头。
        let disk_root = dir.path().join("antigravity");
        let main_exe = disk_root.join("Antigravity.exe");
        let bin_dir = disk_root.join("resources").join("bin");
        let server = bin_dir.join("language_server_x64.exe");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(&main_exe, []).unwrap();
        std::fs::write(&server, []).unwrap();
        let root = dir.path().join("Antigravity");
        let install = Install {
            main_exe: root.join("Antigravity.exe"),
            bin_dir: root.join("resources").join("bin"),
            root,
        };
        assert_eq!(
            classify_in_root(&main_exe, &install.root, &install.main_exe, false),
            None
        );
        assert_eq!(
            classify_executable(&install, &main_exe),
            Some(ProcessKind::App)
        );
        assert_eq!(
            classify_executable(&install, &main_exe.canonicalize().unwrap()),
            Some(ProcessKind::App),
        );
        assert_eq!(
            classify_executable(&install, &server.canonicalize().unwrap()),
            Some(ProcessKind::LanguageServer),
        );
    }

    #[cfg(windows)]
    #[test]
    fn queries_the_executable_image_of_a_running_windows_process() {
        let path = windows_image_path(std::process::id()).unwrap();
        assert_eq!(
            path.canonicalize().unwrap(),
            std::env::current_exe().unwrap().canonicalize().unwrap(),
        );
    }
}
