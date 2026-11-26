//! Tauri commands: the frontend's only entry points. Each one delegates to
//! the module that owns the behavior.

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::antigravity::process;
use crate::error::{Error, Result};
use crate::model::{RelatedProcess, Snapshot};
use crate::network::{self, NetworkSettings};
use crate::state::Orbit;
use crate::{accounts, autostart, quota, scheduler, shell, switcher};

type Shared<'a> = State<'a, Arc<Orbit>>;

/// Quotas younger than this are not refetched when the surface opens.
const STALE_AFTER: Duration = Duration::from_secs(60);

#[tauri::command]
pub fn get_snapshot(orbit: Shared<'_>) -> Snapshot {
    orbit.snapshot()
}

#[tauri::command]
pub async fn refresh(orbit: Shared<'_>, force: bool) -> Result<()> {
    if force {
        scheduler::tick(&orbit).await;
    } else {
        quota::refresh_if_stale(&orbit, STALE_AFTER).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn test_network(settings: NetworkSettings) -> Result<()> {
    network::test_connection(settings).await
}

#[tauri::command]
pub async fn save_network(orbit: Shared<'_>, settings: NetworkSettings) -> Result<()> {
    let sync = orbit.sync_lock.try_lock().map_err(|_| Error::Busy)?;
    if !orbit.is_idle() {
        return Err(Error::Busy);
    }
    orbit.save_network(settings)?;
    drop(sync);
    let orbit = Arc::clone(&orbit);
    tauri::async_runtime::spawn(async move { scheduler::tick(&orbit).await });
    Ok(())
}
#[tauri::command]
pub async fn add_account(orbit: Shared<'_>) -> Result<String> {
    let result = accounts::add(&orbit).await;
    if let Err(err) = &result
        && !matches!(err, Error::OAuthCancelled)
    {
        log::warn!("could not add Antigravity account: {err}");
    }
    result
}

#[tauri::command]
pub fn cancel_add_account(orbit: Shared<'_>) {
    accounts::cancel_add(&orbit);
}

#[tauri::command]
pub fn copy_sign_in_link(orbit: Shared<'_>) -> Result<()> {
    accounts::copy_sign_in_link(&orbit)
}

#[tauri::command]
pub async fn remove_account(orbit: Shared<'_>, id: String) -> Result<()> {
    accounts::remove(&orbit, &id).await
}

#[tauri::command]
pub async fn switch_account(orbit: Shared<'_>, id: String) -> Result<()> {
    switcher::switch(&orbit, &id).await
}

#[tauri::command]
pub fn running_processes(orbit: Shared<'_>) -> Vec<RelatedProcess> {
    process::scan(orbit.install().ok().as_ref())
}

#[tauri::command]
pub async fn stop_client(orbit: Shared<'_>) -> Result<()> {
    switcher::stop(&orbit).await
}

#[tauri::command]
pub async fn restart_client(orbit: Shared<'_>) -> Result<()> {
    switcher::restart(&orbit).await
}

#[tauri::command]
pub fn resize_panel(app: AppHandle, height: f64) {
    shell::resize(&app, height);
}

#[tauri::command]
pub fn reveal_logs(app: AppHandle) -> Result<()> {
    let dir = app
        .path()
        .app_log_dir()
        .map_err(|err| Error::Invalid(err.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|err| Error::Invalid(err.to_string()))
}

#[tauri::command]
pub fn autostart_enabled() -> Result<bool> {
    autostart::is_enabled()
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<()> {
    autostart::set_enabled(enabled)
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}
