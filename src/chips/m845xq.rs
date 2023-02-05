use crate::{
    chips::AccelerometerChip, utils, AccelerometerError, AccelerometerResult, DeviceConfig,
    OutputDataRate, Scale, Value,
};
use embedded_hal::blocking::i2c;
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;
use std::{cell::RefCell, thread};

struct ChipConstants;
impl ChipConstants {
    const DEFAULT_I2C_ADDRESS: u8 = 0x1d;
    const OUT_X_MSB: u8 = 0x01;

    const XYZ_DATA_CFG: u8 = 0x0e;

    const CTRL_REG1: u8 = 0x2a;

    const TWO_G_SCALE_FACTOR: f64 = utils::G_METERS_PER_SECOND / 1024.0;
    const FOUR_G_SCALE_FACTOR: f64 = utils::G_METERS_PER_SECOND / 512.0;
    const EIGHT_G_SCALE_FACTOR: f64 = utils::G_METERS_PER_SECOND / 256.0;

    const TWO_G_CFG_BITS: u8 = 0b00;
    const FOUR_G_CFG_BITS: u8 = 0b01;
    const EIGHT_G_CFG_BITS: u8 = 0b10;
}

fn apply_scale(scale: &Scale, value: i16) -> f64 {
    match scale {
        Scale::TwoG => ChipConstants::TWO_G_SCALE_FACTOR * value as f64,
        Scale::FourG => ChipConstants::FOUR_G_SCALE_FACTOR * value as f64,
        Scale::EightG => ChipConstants::EIGHT_G_SCALE_FACTOR * value as f64,
        _ => 0.0,
    }
}

pub struct M845xQImpl<I2C> {
    /// The concrete I²C device implementation.
    i2c: RefCell<I2C>,

    address: u8,

    scale: Scale,

    data_rate: OutputDataRate,
}

impl<I2C> M845xQImpl<I2C>
where
    I2C: i2c::Write<Error = LinuxI2CError> + i2c::WriteRead<Error = LinuxI2CError>,
{
    pub(crate) fn new(i2c: I2C, config: &DeviceConfig) -> Self {
        let mut value = M845xQImpl {
            i2c: RefCell::new(i2c),
            address: config.address.unwrap_or(ChipConstants::DEFAULT_I2C_ADDRESS),
            scale: config.scale,
            data_rate: config.data_rate,
        };

        value
            .update_scale()
            .and_then(|_| value.update_data_rate())
            .and_then(|_| value.delay_for_update())
            .unwrap();

        value
    }

    fn update_scale(&mut self) -> Result<(), AccelerometerError> {
        // Implementation decision: This function name assumes we are only ever
        // going to update the scale in XYZ_DATA_CFG... we deliberately leave
        // HPF_OUT unchanged.

        let scale_bits = match self.scale {
            Scale::TwoG => ChipConstants::TWO_G_CFG_BITS,
            Scale::FourG => ChipConstants::FOUR_G_CFG_BITS,
            Scale::EightG => ChipConstants::EIGHT_G_CFG_BITS,
            _ => {
                return Err(AccelerometerError::NotSupportedByChip);
            }
        };

        self.standby()
            .and_then(|_| self.write_xyz_data_cfg(scale_bits))
            .and_then(|_| self.active())
            .and_then(|_| self.delay_for_update())
            .and(Ok(()))
    }

    fn update_data_rate(&mut self) -> Result<(), AccelerometerError> {
        // Implementation decision: This function name assumes we are only ever
        // going to update the data rate in CTRL_REG1... we deliberately leave
        // ASLP_RATE, LNOISE, and F_READ unchanged.

        let original_ctrl_reg1 = self.read_ctrl_reg1()?;
        let data_rate_mask = (self.data_rate.clone() as u8) << 3;
        let updated_ctrl_reg1 = original_ctrl_reg1 & 0b11000111;
        let updated_ctrl_reg1 = updated_ctrl_reg1 | data_rate_mask;

        // Per the documentation: Except for STANDBY mode selection, the device
        // must be in STANDBY mode to change any of the fields within CTRL_REG1
        self.standby()
            .and_then(|_| self.write_ctrl_reg1(updated_ctrl_reg1))
            .and_then(|_| self.active())
            .and(Ok(()))
    }

    fn read_ctrl_reg1(&self) -> Result<u8, AccelerometerError> {
        let mut data = [0];

        self.i2c
            .borrow_mut()
            .write_read(self.address, &[ChipConstants::CTRL_REG1], &mut data)
            .map_err(AccelerometerError::I2CBusError)
            .and(Ok(data[0]))
    }

    fn write_xyz_data_cfg(&mut self, value: u8) -> Result<(), AccelerometerError> {
        self.i2c
            .borrow_mut()
            .write(self.address, &[ChipConstants::XYZ_DATA_CFG, value])
            .map_err(AccelerometerError::I2CBusError)
            .and(Ok(()))
    }

    fn write_ctrl_reg1(&mut self, value: u8) -> Result<(), AccelerometerError> {
        self.i2c
            .borrow_mut()
            .write(self.address, &[ChipConstants::CTRL_REG1, value])
            .map_err(AccelerometerError::I2CBusError)
            .and(Ok(()))
    }

    fn to_meters_per_second(&self, buffer: &[u8]) -> f64 {
        // The most significant 8-bits of each axis are stored in the MSB register
        // (explains the right-shift by 4-bits)
        let value = (((buffer[0] as i16) << 8) | buffer[1] as i16) >> 4;

        apply_scale(&self.scale, value)
    }

    fn standby(&mut self) -> AccelerometerResult<()> {
        let original_ctrl_reg1 = self.read_ctrl_reg1()?;
        let updated_ctrl_reg1 = original_ctrl_reg1 & 0b11111110;
        self.write_ctrl_reg1(updated_ctrl_reg1)
    }

    fn active(&mut self) -> AccelerometerResult<()> {
        let original_ctrl_reg1 = self.read_ctrl_reg1()?;
        let updated_ctrl_reg1 = original_ctrl_reg1 | 0b00000001;
        self.write_ctrl_reg1(updated_ctrl_reg1)
    }

    fn delay_for_update(&self) -> AccelerometerResult<()> {
        self.data_rate
            .update_cycle_duration()
            .checked_mul(2)
            .map(|value| {
                thread::sleep(value);
            });
        Ok(())
    }
}

impl<I2C> AccelerometerChip for M845xQImpl<I2C>
where
    I2C: i2c::Write<Error = LinuxI2CError> + i2c::WriteRead<Error = LinuxI2CError>,
{
    fn default_chip_address(&self) -> u8 {
        ChipConstants::DEFAULT_I2C_ADDRESS
    }

    fn average(&self, sample_count: u8) -> AccelerometerResult<Value> {
        let mut avg: Value = Default::default();

        for c in 0..sample_count {
            let m = self.raw_measurement().unwrap();

            log::debug!(target: "acclrmtr", "Zero sample {}: {:?}", c, m);
            avg.mut_add(&m);

            self.delay_for_update().unwrap();
        }

        avg.mut_div(sample_count as f64);

        Ok(avg)
    }

    fn raw_measurement(&self) -> AccelerometerResult<Value> {
        let mut data: [u8; 6] = [0; 6];

        self.i2c
            .borrow_mut()
            .write_read(self.address, &[ChipConstants::OUT_X_MSB], &mut data)
            .map_err(AccelerometerError::I2CBusError)
            .and(Ok(Value {
                x: self.to_meters_per_second(&data[0..2]),
                y: self.to_meters_per_second(&data[2..4]),
                z: self.to_meters_per_second(&data[4..6]),
            }))
    }
}
