use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strokers_core::AxisKind;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RootConfig {
    pub stroker: StrokerConfig,
    pub limits: BTreeMap<AxisKind, LimitsConfig>,
}

/// Specify how to connect to the stroker.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StrokerConfig {
    /// Connect over a serial port and control with [T-Code] commands
    ///
    /// [T-Code]: https://github.com/multiaxis/TCode-Specification
    #[serde(rename = "tcode_serial")]
    TCodeSerial {
        /// The serial port for the T-Code device.
        /// On Linux: often /dev/ttyUSB0
        /// On Windows: this looks like COM5 or with some other number
        serial_port: String,

        /// The baud rate used for the serial port.
        /// Defaults to 115200.
        #[serde(default = "default_tcode_baud_rate")]
        baud: u32,
    },

    /// Don't connect to a stroker, just emit debug information to the log.
    #[serde(rename = "debug")]
    Debug,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Speed limit in full-scales per second
    pub speed: f32,

    /// Default minimum limit of the axis.
    /// Note that this can often be controlled dynamically later on.
    pub default_min: f32,

    /// Default maximum limit of the axis.
    /// Note that this can often be controlled dynamically later on.
    pub default_max: f32,
}

fn default_tcode_baud_rate() -> u32 {
    115200
}
