use crate::{AccelerometerMeasurement, DeviceConfig, OutputDataRate, Scale};
use chrono::{DateTime, Utc};
use std::{
    fmt,
    time::{self, SystemTime},
};

pub const G_METERS_PER_SECOND: f64 = 9.81;

/// A (mockable) clock entity
pub(crate) trait Clock {
    /// Returns the current [SystemTime]
    fn now(&self) -> SystemTime;
}

/// An implementation of [Clock] which uses [SystemTime]
pub(crate) struct SystemTimeClock;
impl Clock for SystemTimeClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

impl Default for Scale {
    fn default() -> Self {
        Scale::FourG
    }
}

impl Default for OutputDataRate {
    fn default() -> Self {
        OutputDataRate::DataRate50Hz
    }
}

impl AccelerometerMeasurement {
    pub fn new_default(time: SystemTime) -> Self {
        AccelerometerMeasurement {
            time: time,
            acceleration: Default::default(),
            estimated_velocity: Default::default(),
        }
    }
}

impl OutputDataRate {
    const DATA_RATE_800HZ_UPDATE_FREQUENCY: f64 = 800.0;
    const DATA_RATE_400HZ_UPDATE_FREQUENCY: f64 = 400.0;
    const DATA_RATE_200HZ_UPDATE_FREQUENCY: f64 = 200.0;
    const DATA_RATE_100HZ_UPDATE_FREQUENCY: f64 = 100.0;
    const DATA_RATE_50HZ_UPDATE_FREQUENCY: f64 = 50.0;
    const DATA_RATE_12_5HZ_UPDATE_FREQUENCY: f64 = 12.5;
    const DATA_RATE_6_25HZ_UPDATE_FREQUENCY: f64 = 6.25;
    const DATA_RATE_1_56HZ_UPDATE_FREQUENCY: f64 = 1.56;

    pub fn update_frequency_hz(&self) -> f64 {
        match &*self {
            OutputDataRate::DataRate800Hz => OutputDataRate::DATA_RATE_800HZ_UPDATE_FREQUENCY,
            OutputDataRate::DataRate400Hz => OutputDataRate::DATA_RATE_400HZ_UPDATE_FREQUENCY,
            OutputDataRate::DataRate200Hz => OutputDataRate::DATA_RATE_200HZ_UPDATE_FREQUENCY,
            OutputDataRate::DataRate100Hz => OutputDataRate::DATA_RATE_100HZ_UPDATE_FREQUENCY,
            OutputDataRate::DataRate50Hz => OutputDataRate::DATA_RATE_50HZ_UPDATE_FREQUENCY,
            OutputDataRate::DataRate12_5Hz => OutputDataRate::DATA_RATE_12_5HZ_UPDATE_FREQUENCY,
            OutputDataRate::DataRate6_25Hz => OutputDataRate::DATA_RATE_6_25HZ_UPDATE_FREQUENCY,
            OutputDataRate::DataRate1_56Hz => OutputDataRate::DATA_RATE_1_56HZ_UPDATE_FREQUENCY,
        }
    }

    pub fn update_cycle_duration(&self) -> time::Duration {
        time::Duration::from_secs_f64(1.0 / &self.update_frequency_hz())
    }
}

impl DeviceConfig {
    pub(crate) fn log_info(&self, default_chip_address: u8) {
        log::info!(target: "acclrmtr", "Chip type:          {:?}", self.chip);
        log::info!(target: "acclrmtr", "I²C Device file:    {}", self.i2c_device_file);
        log::info!(target: "acclrmtr", "Full scale mode:    {:?}", self.scale);
        log::info!(target: "acclrmtr", "Data rate:          {:?}", self.data_rate);
        log::info!(target: "acclrmtr", "Address:            {:#02x}", self.address.unwrap_or(default_chip_address));
    }
}

impl fmt::Display for AccelerometerMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let time: DateTime<Utc> = self.time.into();

        write!(
            f,
            "{}: Acc. (m/s²): {:>9.5}, {:>9.5}, {:>9.5}",
            time.to_rfc3339(),
            self.acceleration.x,
            self.acceleration.y,
            self.acceleration.z,
        )
    }
}

impl fmt::Debug for AccelerometerMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let time: DateTime<Utc> = self.time.into();

        match self.estimated_velocity {
            Some(velocity) => {
                write!(
                    f,
                    "{}: Acc. (m/s²): {:>9.5}, {:>9.5}, {:>9.5} [Est. vel. (m/s): {:>9.5}, {:>9.5}, {:>9.5}]",
                    time.format("%F %H:%M:%S.%3f"),
                    self.acceleration.x,
                    self.acceleration.y,
                    self.acceleration.z,
                    velocity.x,
                    velocity.y,
                    velocity.z
                )
            }
            None => {
                // Use fmt::Display instead
                write!(f, "{}", self)
            }
        }
    }
}
