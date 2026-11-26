//! OrbitSwitch 的网络偏好和不带账号凭据的连通性检查。

use std::num::NonZeroU16;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::google::{USER_AGENT, oauth::USERINFO_ENDPOINT};
use crate::registry::write_file;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkSettings {
    pub local_http_proxy_port: Option<NonZeroU16>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    pub local_http_proxy_port: Option<NonZeroU16>,
    pub invalid: bool,
}

pub struct Network {
    path: PathBuf,
    settings: NetworkSettings,
    client: Option<reqwest::Client>,
}

impl Network {
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("network.json");
        let settings = match std::fs::read(&path) {
            Ok(raw) => serde_json::from_slice(&raw).ok(),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Some(NetworkSettings::default())
            }
            Err(_) => None,
        };
        let client = settings.and_then(|settings| build_client(settings).ok());
        if client.is_none() {
            log::warn!("OrbitSwitch network configuration could not be loaded");
        } else if let Some(port) = settings.and_then(|settings| settings.local_http_proxy_port) {
            log::info!("using configured loopback HTTPS proxy on port {port}");
        } else {
            log::info!("using system network");
        }
        Self {
            path,
            settings: settings.unwrap_or_default(),
            client,
        }
    }

    pub fn status(&self) -> NetworkStatus {
        NetworkStatus {
            local_http_proxy_port: self.settings.local_http_proxy_port,
            invalid: self.client.is_none(),
        }
    }

    pub fn client(&self) -> Result<reqwest::Client> {
        self.client.clone().ok_or(Error::NetworkConfiguration)
    }

    pub fn save(&mut self, settings: NetworkSettings) -> Result<()> {
        let client = build_client(settings)?;
        if settings.local_http_proxy_port.is_some() {
            write_file(&self.path, &serde_json::to_vec_pretty(&settings)?)?;
        } else if let Err(err) = std::fs::remove_file(&self.path)
            && err.kind() != std::io::ErrorKind::NotFound
        {
            return Err(err.into());
        }
        self.settings = settings;
        self.client = Some(client);
        Ok(())
    }
}

fn build_client(settings: NetworkSettings) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30));
    if let Some(port) = settings.local_http_proxy_port {
        builder = builder.proxy(reqwest::Proxy::https(format!("http://127.0.0.1:{port}"))?);
    }
    Ok(builder.build()?)
}

pub async fn test_connection(settings: NetworkSettings) -> Result<()> {
    let response = build_client(settings)?
        .get(USERINFO_ENDPOINT)
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Ok(());
    }
    Err(Error::Http {
        endpoint: "oauth2/userinfo",
        status: response.status().as_u16(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_existing_port_and_recovers_from_a_broken_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("network.json");
        std::fs::write(&path, br#"{"localHttpProxyPort":10808}"#).unwrap();
        let network = Network::load(dir.path());
        assert_eq!(
            network.status().local_http_proxy_port,
            NonZeroU16::new(10808)
        );
        assert!(!network.status().invalid);

        std::fs::write(&path, br#"{"localHttpProxyPort":0}"#).unwrap();
        let mut network = Network::load(dir.path());
        assert!(network.status().invalid);
        assert!(matches!(network.client(), Err(Error::NetworkConfiguration)));

        network.save(NetworkSettings::default()).unwrap();
        assert!(!path.exists());
        assert!(network.client().is_ok());
    }
}
