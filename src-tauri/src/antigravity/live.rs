//! The credential Antigravity is currently signed in with.
//!
//! Antigravity's language server stores it with go-keyring under service
//! `gemini`, account `antigravity`, and mirrors it to
//! `~/.gemini/jetski-standalone-oauth-token`, which it reads when the keyring
//! times out. OrbitSwitch reads the keyring first and verifies both on switch.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::antigravity::credential::Bundle;
use crate::error::{Error, Result};
use crate::secret_store;
use crate::transaction;

const SERVICE: &str = "gemini";
const ACCOUNT: &str = "antigravity";

fn fallback_path() -> Result<PathBuf> {
    std::env::home_dir()
        .map(|home| home.join(".gemini").join("jetski-standalone-oauth-token"))
        .ok_or_else(|| Error::Invalid("home directory is unknown".into()))
}

pub fn read() -> Result<Option<Bundle>> {
    if let Some(raw) = secret_store::get(SERVICE, ACCOUNT)? {
        return Bundle::decode(&raw).map(Some);
    }
    read_fallback(&fallback_path()?)?
        .as_deref()
        .map(Bundle::decode)
        .transpose()
}

fn read_fallback(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[derive(PartialEq, Eq)]
struct CredentialCopies {
    keyring: Option<String>,
    fallback: Option<String>,
}

impl CredentialCopies {
    fn read(path: &Path, read_keyring: &impl Fn() -> Result<Option<String>>) -> Result<Self> {
        Ok(Self {
            keyring: read_keyring()?,
            fallback: read_fallback(path)?,
        })
    }

    fn write_fallback(&self, path: &Path) -> Result<()> {
        match &self.fallback {
            Some(value) => write_private_file(path, value.as_bytes()),
            None => match fs::remove_file(path) {
                Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(err.into()),
                _ => Ok(()),
            },
        }
    }

    fn verify(
        &self,
        path: &Path,
        read_keyring: &impl Fn() -> Result<Option<String>>,
    ) -> Result<()> {
        if &Self::read(path, read_keyring)? == self {
            Ok(())
        } else {
            Err(Error::VerifyFailed)
        }
    }
}

/// 仅在 Antigravity 完全退出后调用。回滚保留每个位置原本的内容或缺失状态。
pub fn replace(bundle: &Bundle) -> Result<()> {
    let path = fallback_path()?;
    let target = CredentialCopies {
        keyring: Some(bundle.to_keyring_value()?),
        fallback: Some(bundle.to_json()?),
    };
    replace_copies(
        &path,
        &target,
        || secret_store::get(SERVICE, ACCOUNT),
        |value| match value {
            Some(value) => secret_store::set(SERVICE, ACCOUNT, value),
            None => secret_store::delete(SERVICE, ACCOUNT),
        },
    )
}

fn replace_copies(
    path: &Path,
    target: &CredentialCopies,
    read_keyring: impl Fn() -> Result<Option<String>>,
    write_keyring: impl Fn(Option<&str>) -> Result<()>,
) -> Result<()> {
    let previous = CredentialCopies::read(path, &read_keyring)?;
    transaction::apply(
        || {
            write_keyring(target.keyring.as_deref())?;
            target.write_fallback(path)?;
            target.verify(path, &read_keyring)
        },
        || {
            log::warn!("Antigravity credential switch failed; restoring the previous copies");
            // 尽力恢复两份凭据，即使其中一个位置失败。
            let keyring = write_keyring(previous.keyring.as_deref());
            let fallback = previous.write_fallback(path);
            keyring.and(fallback)?;
            previous.verify(path, &read_keyring)
        },
    )
}

/// 原子替换文件，临时文件只允许当前用户访问。
fn write_private_file(path: &Path, contents: &[u8]) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| Error::Invalid("credential path has no parent".into()))?;
    fs::create_dir_all(dir)?;
    let mut nonce = [0; 8];
    getrandom::fill(&mut nonce).map_err(|err| Error::Invalid(err.to_string()))?;
    let tmp = dir.join(format!(
        ".orbitswitch-{:016x}.tmp",
        u64::from_ne_bytes(nonce)
    ));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&tmp)?;
    let result = (|| -> std::io::Result<()> {
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&tmp, path)
    })();
    result.inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn failed_readback_restores_keyring_and_original_fallback_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("credential");
        for original in [None, Some("original file\n")] {
            let keyring = RefCell::new(Some("original keyring".to_owned()));
            let read = || Ok(keyring.borrow().clone());
            if let Some(raw) = original {
                fs::write(&path, raw).unwrap();
            }
            let target = CredentialCopies {
                keyring: Some("target keyring".into()),
                fallback: Some("target file".into()),
            };
            let result = replace_copies(&path, &target, read, |raw| {
                let raw = raw.map(|value| {
                    if value == "target keyring" {
                        "wrong readback"
                    } else {
                        value
                    }
                    .to_owned()
                });
                *keyring.borrow_mut() = raw;
                Ok(())
            });
            assert!(matches!(result, Err(Error::VerifyFailed)));
            assert_eq!(keyring.borrow().as_deref(), Some("original keyring"));
            assert_eq!(read_fallback(&path).unwrap().as_deref(), original);
        }
    }
}
