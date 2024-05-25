//! # strokers
//!
//! This library lets you control strokers (adult toys).
//!
//! Currently this library only supports strokers that implement
//! the T-Code protocol over a serial port,
//! such as the [Tempest MAx] OSR2, OSR2+, SR6, SSR1 and similar derivatives.
//!
//! (These machines are self-built using a 3D printer and relatively accessible hardware.
//! The OSR2 is completely open source but the other machines' designs are available
//! behind a modest paywall to his members only.)
//!
//!
//! [Tempest MAx]: https://www.patreon.com/tempestvr

use std::path::{Path, PathBuf};

use config::{RootConfig, StrokerConfig};
use devices::AnyStroker;
use eyre::ContextCompat;
pub use strokers_core as core;
use strokers_device_tcode::SerialTCodeStroker;
use thiserror::Error;

pub mod config;

pub mod devices;

#[derive(Debug, Error)]
pub enum StrokersError {
    #[error("i/o error: {0}")]
    IoError(tokio::io::Error),

    #[error("failed to deserialise TOML config at {1:?}: {0}")]
    ConfigDeserialisationError(toml::de::Error, PathBuf),

    #[error("failed to connect to stroker: {0:?}")]
    Connection(eyre::Error),

    #[error("unexpected error: {0:?}")]
    Unexpected(eyre::Error),
}

/// Load the Strokers configuration from the default place.
///
/// On Linux this is at `~/.config/strokers.toml`.
///
/// In any case the environment variable `STROKERS_CONFIG` overrides this location.
pub async fn load_config() -> Result<RootConfig, StrokersError> {
    if let Ok(env_var) = std::env::var("STROKERS_CONFIG") {
        let path = PathBuf::from(env_var);
        load_config_from_path(&path).await
    } else {
        let config_dir = dirs::config_dir()
            .context("can't find config_dir()")
            .map_err(StrokersError::Unexpected)?;
        let path = config_dir.join("strokers.toml");
        load_config_from_path(&path).await
    }
}

/// Load the Strokers configuration from the given path.
///
/// Use [`load_config`] to use the default path.
pub async fn load_config_from_path(path: &Path) -> Result<RootConfig, StrokersError> {
    let text = tokio::fs::read_to_string(path)
        .await
        .map_err(StrokersError::IoError)?;
    toml::from_str(&text)
        .map_err(|toml_err| StrokersError::ConfigDeserialisationError(toml_err, path.to_owned()))
}

/// Attempt to open a stroker from its configuration.
pub async fn open_stroker(config: &StrokerConfig) -> Result<AnyStroker, StrokersError> {
    match config {
        StrokerConfig::TCodeSerial { serial_port, baud } => {
            let stroker = SerialTCodeStroker::connect(serial_port, *baud)
                .await
                .map_err(StrokersError::Connection)?;
            Ok(AnyStroker::new(stroker))
        }
        StrokerConfig::Debug => todo!(),
    }
}
