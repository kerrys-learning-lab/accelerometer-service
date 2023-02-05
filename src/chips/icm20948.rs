use std::{cell::RefCell, thread, time};

use embedded_hal::blocking::i2c;
use linux_embedded_hal::i2cdev::linux::LinuxI2CError;

use crate::{utils, AccelerometerError, AccelerometerResult, DeviceConfig, Scale, Value};

use super::AccelerometerChip;

struct ChipConstants;
impl ChipConstants {
    const DEFAULT_I2C_ADDRESS: u8 = 0x68;

    const WHO_AM_I: u8 = 0x00;
    const PWR_MGMT_1: u8 = 0x06;
    const _PWR_MGMT_2: u8 = 0x06;
    const ACCEL_CFG: u8 = 0x14;
    const ACCEL_XOUT_H: u8 = 0x2d;
    const REG_BANK_SEL: u8 = 0x7f;

    const WHO_SHOULD_I_BE: u8 = 0xea;
    const USER_BANK_0: u8 = 0b00000000;
    const _USER_BANK_1: u8 = 0b00010000;
    const USER_BANK_2: u8 = 0b00100000;
    const _USER_BANK_3: u8 = 0b00110000;

    const TWO_G_SCALE_FACTOR: f64 = utils::G_METERS_PER_SECOND / 16384.0;
    const FOUR_G_SCALE_FACTOR: f64 = utils::G_METERS_PER_SECOND / 8192.0;
    const EIGHT_G_SCALE_FACTOR: f64 = utils::G_METERS_PER_SECOND / 4096.0;
    const SIXTEEN_G_SCALE_FACTOR: f64 = utils::G_METERS_PER_SECOND / 2048.0;

    const PWR_MGMT_1_RESET_BITS: u8 = 0b10000000;
    const PWR_MGMT_1_ENABLE_BITS: u8 = 0b00000001;

    const TWO_G_CFG_BITS: u8 = 0b00000000;
    const FOUR_G_CFG_BITS: u8 = 0b00000010;
    const EIGHT_G_CFG_BITS: u8 = 0b00000100;
    const SIXTEEN_G_CFG_BITS: u8 = 0b00000110;
    const ACCEL_DLPCFG_BITS: u8 = 0b00110000; // Mode 6
    const ACCEL_FCHOICE_BITS: u8 = 0b00000001; // Enable digital low-pass filter (DLPF)
}

const LOG_TARGET: &'static str = "icm20948";

pub(crate) struct Icm20948Impl<I2C> {
    i2c: RefCell<I2C>,
    address: u8,
    scale: Scale,
}

fn apply_scale(scale: &Scale, value: i16) -> f64 {
    match scale {
        Scale::TwoG => ChipConstants::TWO_G_SCALE_FACTOR * value as f64,
        Scale::FourG => ChipConstants::FOUR_G_SCALE_FACTOR * value as f64,
        Scale::EightG => ChipConstants::EIGHT_G_SCALE_FACTOR * value as f64,
        Scale::SixteenG => ChipConstants::SIXTEEN_G_SCALE_FACTOR * value as f64,
    }
}

impl<I2C> Icm20948Impl<I2C>
where
    I2C: i2c::Write<Error = LinuxI2CError> + i2c::WriteRead<Error = LinuxI2CError>,
{
    pub(crate) fn new(i2c: I2C, config: &DeviceConfig) -> Self {
        let mut chip = Icm20948Impl {
            i2c: RefCell::new(i2c),
            address: config.address.unwrap_or(ChipConstants::DEFAULT_I2C_ADDRESS),
            scale: config.scale,
        };

        chip.verify_identity()
            .and_then(|_| chip.reset())
            .and_then(|_| {
                thread::sleep(time::Duration::from_secs_f64(0.1));
                Ok(())
            })
            .and_then(|_| chip.enable())
            .and_then(|_| chip.update_scale())
            //
            // Important! Default to USER_BANK_0 for subsequent reads of ACCEL_XOUT_H
            .and_then(|_| chip.select_user_bank(ChipConstants::USER_BANK_0))
            .unwrap();

        chip
    }

    fn verify_identity(&mut self) -> AccelerometerResult<()> {
        self.select_user_bank(ChipConstants::USER_BANK_0)
            .and_then(|_| self.read_register(ChipConstants::WHO_AM_I))
            .and_then(|who_am_i| {
                if who_am_i != ChipConstants::WHO_SHOULD_I_BE {
                    log::error!(
                        target: LOG_TARGET,
                        "Identity crisis!  I should be {:#04x}, but I'm actually {:#04x}",
                        ChipConstants::WHO_SHOULD_I_BE,
                        who_am_i
                    );
                    Err(AccelerometerError::InvalidInputDataError)
                } else {
                    Ok(())
                }
            })
    }

    fn select_user_bank(&mut self, bank: u8) -> AccelerometerResult<()> {
        log::debug!(target: LOG_TARGET, "REG_BANK_SEL: {}", bank >> 4);

        self.write_register(ChipConstants::REG_BANK_SEL, bank)
            .and(Ok(()))
    }

    fn reset(&mut self) -> AccelerometerResult<()> {
        self.select_user_bank(ChipConstants::USER_BANK_0)
            .and_then(|_| {
                self.write_register(
                    ChipConstants::PWR_MGMT_1,
                    ChipConstants::PWR_MGMT_1_RESET_BITS,
                )
            })
            .and_then(|updated_value| {
                log::debug!(
                    target: LOG_TARGET,
                    "Updated PWR_MGMT_1 value: {:#10b}",
                    updated_value
                );

                Ok(())
            })
    }

    fn enable(&mut self) -> AccelerometerResult<()> {
        self.select_user_bank(ChipConstants::USER_BANK_0)
            .and_then(|_| {
                self.write_register(
                    ChipConstants::PWR_MGMT_1,
                    ChipConstants::PWR_MGMT_1_ENABLE_BITS,
                )
            })
            .and_then(|updated_value| {
                log::debug!(
                    target: LOG_TARGET,
                    "Updated PWR_MGMT_1 value: {:#10b}",
                    updated_value
                );

                Ok(())
            })
    }

    fn read_register(&self, register: u8) -> AccelerometerResult<u8> {
        let mut data: [u8; 1] = [0];

        self.i2c
            .borrow_mut()
            .write_read(self.address, &[register], &mut data)
            .unwrap();

        Ok(data[0])
    }

    fn write_register(&mut self, register: u8, value: u8) -> AccelerometerResult<u8> {
        let mut i2c = self.i2c.borrow_mut();

        i2c.write(self.address, &[register, value])
            .map_err(AccelerometerError::I2CBusError)
            .and_then(|_| {
                thread::sleep(time::Duration::from_secs_f64(0.1));
                Ok(())
            })
            .and_then(|_| {
                // NOTE: Can't call "self.read_register", as it will re-borrow,
                //       causing a panic
                let mut data: [u8; 1] = [0];

                i2c.write_read(self.address, &[register], &mut data)
                    .map_err(AccelerometerError::I2CBusError)
                    .and(Ok(data[0]))
            })
    }

    fn update_scale(&mut self) -> AccelerometerResult<()> {
        // Implementation decision: This function name assumes we are only ever
        // going to update the scale in ACCEL_CONFIG... we deliberately leave
        // ACCEL_DLPFCFG and ACCEL_FCHOICE unchanged.

        let scale_bits = match self.scale {
            Scale::TwoG => ChipConstants::TWO_G_CFG_BITS,
            Scale::FourG => ChipConstants::FOUR_G_CFG_BITS,
            Scale::EightG => ChipConstants::EIGHT_G_CFG_BITS,
            Scale::SixteenG => ChipConstants::SIXTEEN_G_CFG_BITS,
        };

        let value =
            scale_bits | ChipConstants::ACCEL_DLPCFG_BITS | ChipConstants::ACCEL_FCHOICE_BITS;

        log::debug!(
            target: LOG_TARGET,
            "ACCEL_CFG (desired): {:#04x} (scale: {})",
            value,
            scale_bits >> 1
        );

        self.select_user_bank(ChipConstants::USER_BANK_2)
            .and_then(|_| self.write_register(ChipConstants::ACCEL_CFG, value))
            .and_then(|updated_value| {
                log::debug!(
                    target: LOG_TARGET,
                    "Updated ACCEL_CFG value: {:#010b}",
                    updated_value
                );

                Ok(())
            })
    }

    fn to_meters_per_second(&self, buffer: &[u8]) -> f64 {
        let value = i16::from_be_bytes([buffer[0], buffer[1]]);

        apply_scale(&self.scale, value)
    }
}

impl<I2C> AccelerometerChip for Icm20948Impl<I2C>
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

            log::debug!(target: LOG_TARGET, "Zero sample {}: {:?}", c, m);
            avg.mut_add(&m);

            thread::sleep(time::Duration::from_secs_f64(0.1));
        }

        avg.mut_div(sample_count as f64);

        Ok(avg)
    }

    fn raw_measurement(&self) -> AccelerometerResult<Value> {
        let mut data: [u8; 6] = [0; 6];

        self.i2c
            .borrow_mut()
            .write_read(self.address, &[ChipConstants::ACCEL_XOUT_H], &mut data)
            .map_err(AccelerometerError::I2CBusError)
            .and(Ok(Value {
                x: self.to_meters_per_second(&data[0..2]),
                y: self.to_meters_per_second(&data[2..4]),
                z: self.to_meters_per_second(&data[4..6]),
            }))
    }
}
