use linux_embedded_hal::I2cdev;

use crate::{AccelerometerResult, DeviceConfig, SupportedChips, Value};

mod icm20948;
mod m845xq;

impl SupportedChips {
    pub(crate) fn new(&self, config: &DeviceConfig) -> Box<dyn AccelerometerChip> {
        let i2c = I2cdev::new(&config.i2c_device_file).unwrap_or_else(|_| {
            panic!("Unable to load I²C device file: {}", config.i2c_device_file)
        });

        let chip: Box<dyn AccelerometerChip> = match &*self {
            SupportedChips::M845xQ => Box::new(m845xq::M845xQImpl::new(i2c, config)),
            SupportedChips::ICM20948 => Box::new(icm20948::Icm20948Impl::new(i2c, config)),
        };

        config.log_info(chip.as_ref().default_chip_address());

        chip
    }
}

pub(crate) trait AccelerometerChip {
    fn default_chip_address(&self) -> u8;

    /// Returns a (current) raw measurement from the accelerometer
    fn raw_measurement(&self) -> AccelerometerResult<Value>;

    fn average(&self, sample_count: u8) -> AccelerometerResult<Value>;
}
