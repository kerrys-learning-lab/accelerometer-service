use linux_embedded_hal::i2cdev::linux::LinuxI2CError;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, time::SystemTime};
use strum::EnumString;

use crate::chips::AccelerometerChip;

pub mod accelerometer;
pub mod chips;
pub mod mqtt;
mod utils;
mod value;

#[derive(Debug, Deserialize, Clone)]
/// An immutable YAML-based configuration of a [Pca9685] device.
pub struct DeviceConfig {
    /// Path to I2C device file (e.g, /dev/i2c-1)
    pub i2c_device_file: String,

    /// Address of PCA9685 (e.g, 0x40)
    pub address: Option<u8>,

    pub chip: SupportedChips,

    #[serde(default)]
    /// Full scale range
    pub scale: Scale,

    #[serde(default)]
    /// Output data rate
    pub data_rate: OutputDataRate,
}

pub struct AccelerometerConfig {}

/// A triple of values for x, y, z.
#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq, Deserialize)]
pub struct Value {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Possible errors in this crate
#[derive(Debug)]
pub enum AccelerometerError {
    /// I²C bus error
    I2CBusError(LinuxI2CError),

    NotSupportedByChip,

    /// Invalid input data provided
    InvalidInputDataError,
}

#[derive(Debug, EnumString, Deserialize, Clone)]
pub enum SupportedChips {
    #[strum(ascii_case_insensitive)]
    M845xQ,

    #[strum(ascii_case_insensitive)]
    ICM20948,
}

/// Customized [Result], where the error type is [AccelerometerError]
pub type AccelerometerResult<T> = Result<T, AccelerometerError>;

#[derive(Debug, EnumString, Deserialize, Clone, Copy)]
pub enum Scale {
    #[strum(ascii_case_insensitive)]
    TwoG,

    #[strum(ascii_case_insensitive)]
    FourG,

    #[strum(ascii_case_insensitive)]
    EightG,

    #[strum(ascii_case_insensitive)]
    SixteenG,
}

#[derive(Debug, EnumString, Deserialize, Clone, Copy)]
pub enum OutputDataRate {
    #[strum(ascii_case_insensitive)]
    DataRate800Hz = 0b000,

    #[strum(ascii_case_insensitive)]
    DataRate400Hz = 0b001,

    #[strum(ascii_case_insensitive)]
    DataRate200Hz = 0b010,

    #[strum(ascii_case_insensitive)]
    DataRate100Hz = 0b011,

    #[strum(ascii_case_insensitive)]
    DataRate50Hz = 0b100,

    #[strum(ascii_case_insensitive)]
    DataRate12_5Hz = 0b101,

    #[strum(ascii_case_insensitive)]
    DataRate6_25Hz = 0b110,

    #[strum(ascii_case_insensitive)]
    DataRate1_56Hz = 0b111,
}

pub struct Accelerometer {
    /// The concrete I²C device implementation.
    chip: Box<dyn AccelerometerChip>,

    previous_measurement: RefCell<AccelerometerMeasurement>,

    clock: Box<dyn utils::Clock>,

    zero: Option<Value>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct AccelerometerMeasurement {
    /// Time at which the measurement was collected
    pub time: SystemTime,

    /// Acceleration, in m/s²
    pub acceleration: Value,

    /// Estimated velocity, in m/s
    pub estimated_velocity: Option<Value>,
}
