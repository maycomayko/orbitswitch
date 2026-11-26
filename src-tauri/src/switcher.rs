//! Switching Antigravity to another account, plus plain stop and restart.
//!
//! Switch order: mint a fresh token for the target first (nothing is touched
//! if Google refuses), save the current sign-in back to its account, stop
//! every Antigravity process, write the credential, verify it, relaunch. If
//! writing or verifying fails, the previous credential is restored; once the
//! new one is verified the switch stands even if the relaunch fails.

use std::sync::Arc;

use crate::antigravity::credential::Bundle;
use crate::antigravity::{live, process};
use crate::error::{Error, Result};
use crate::google::oauth;
use crate::model::{Operation, QuotaState, SwitchStep};
use crate::registry;
use crate::state::Orbit;
use crate::{quota, vault};

pub async fn switch(orbit: &Arc<Orbit>, target_id: &str) -> Result<()> {
    let step = |step| Operation::Switching {
        target_id: target_id.to_owned(),
        step,
    };
    let guard = orbit.begin(step(SwitchStep::Preparing))?;
    let sync = orbit.sync_lock.lock().await;
    let install = orbit.install()?;
    if orbit.state().registry.get(target_id).is_none() {
        return Err(Error::AccountNotFound);
    }
    let refresh_token = vault::get(target_id)?.ok_or(Error::AccountNotFound)?;
    let client = orbit.oauth_client().await?;
    let grant = match oauth::refresh(&orbit.http, &client, &refresh_token).await {
        Err(Error::TokenRevoked) => {
            orbit.update(|state| {
                state
                    .quotas
                    .insert(target_id.to_owned(), QuotaState::Reauth)
            });
            return Err(Error::TokenRevoked);
        }
        other => other?,
    };

    let target =
        oauth::profile(&orbit.http, &grant.access_token, grant.id_token.as_deref()).await?;
    if target.id != target_id {
        return Err(Error::VerifyFailed);
    }

    let bundle = Bundle::from_grant(&grant, &refresh_token, &target)?;
    if bundle
        .claims()
        .is_some_and(|claims| claims.sub != target_id)
    {
        return Err(Error::VerifyFailed);
    }
    save_back(orbit, live::read()?.as_ref()).await?;

    orbit.set_operation(step(SwitchStep::Closing));
    process::stop_all(Some(&install)).await?;

    orbit.set_operation(step(SwitchStep::Writing));
    if let Err(err) = live::replace(&bundle) {
        if !matches!(err, Error::RollbackFailed) {
            let _ = process::launch(&install).await;
        }
        return Err(err);
    }
    orbit
        .tokens
        .put(target_id, grant.access_token.clone(), grant.expires_in);
    orbit.update(|state| -> Result<()> {
        state.active_id = Some(target_id.to_owned());
        state
            .registry
            .set_live_fingerprint(registry::fingerprint(&refresh_token))
    })?;
    log::info!("switched Antigravity to another account");

    orbit.set_operation(step(SwitchStep::Launching));
    let launched = process::launch(&install).await.map_err(Error::after_switch);
    drop(sync);
    drop(guard);
    orbit.refresh_status();
    quota::refresh_one(orbit, target_id).await;
    launched
}

/// Antigravity may hold a newer refresh token for the account it is signed
/// in with (for example after signing in again inside the IDE). Keep it.
async fn save_back(orbit: &Orbit, previous: Option<&Bundle>) -> Result<()> {
    let Some(bundle) = previous.filter(|bundle| !bundle.token.refresh_token.is_empty()) else {
        return Ok(());
    };
    let profile = match bundle.profile(&orbit.http).await {
        Ok(profile) => profile,
        Err(error) => {
            log::warn!("could not identify the outgoing Antigravity sign-in: {error}");
            return Ok(());
        }
    };
    if orbit.state().registry.get(&profile.id).is_none() {
        return Ok(());
    }
    vault::set(&profile.id, &bundle.token.refresh_token)
}

pub async fn stop(orbit: &Arc<Orbit>) -> Result<()> {
    let _guard = orbit.begin(Operation::Stopping)?;
    let install = orbit.install().ok();
    let result = process::stop_all(install.as_ref()).await;
    orbit.refresh_status();
    result
}

pub async fn restart(orbit: &Arc<Orbit>) -> Result<()> {
    let _guard = orbit.begin(Operation::Restarting)?;
    let install = orbit.install()?;
    process::stop_all(Some(&install)).await?;
    let result = process::launch(&install).await;
    orbit.refresh_status();
    result
}
