use std::{cell::RefCell, time::SystemTime};

use crate::{
    utils::Clock, utils::SystemTimeClock, Accelerometer, AccelerometerMeasurement,
    AccelerometerResult, DeviceConfig, Value,
};

impl Accelerometer {
    /// Create a new instance of the device.
    pub fn new(config: &DeviceConfig) -> Self {
        let clock = SystemTimeClock {};
        Accelerometer {
            chip: config.chip.new(config),
            previous_measurement: RefCell::new(AccelerometerMeasurement::new_default(clock.now())),
            clock: Box::new(clock),
            zero: None,
        }
    }

    #[cfg(test)]
    fn mocked(handle: Box<dyn crate::chips::AccelerometerChip>, clock: Box<dyn Clock>) -> Self {
        Accelerometer {
            chip: handle,
            previous_measurement: RefCell::new(AccelerometerMeasurement::new_default(clock.now())),
            clock: clock,
            zero: None,
        }
    }

    pub fn auto_set_zero(&mut self) -> AccelerometerResult<Value> {
        let count = 5;
        log::debug!(target: "acclrmtr",
            "Calculating new zero from average of {} measurements",
            count
        );

        self.chip.average(count).and_then(|avg_measurement| {
            log::debug!(target: "acclrmtr", "Calculated zero: {:?}", avg_measurement);

            self.zero = Some(avg_measurement);

            Ok(avg_measurement)
        })
    }

    pub fn measurement(&self) -> AccelerometerResult<AccelerometerMeasurement> {
        self.get_calibrated_sample().and_then(|value| {
            let now = self.clock.as_ref().now();
            let update = AccelerometerMeasurement {
                time: now,
                acceleration: value,
                estimated_velocity: None, // self.estimate_velocity(now, value),
            };
            self.previous_measurement.replace(update);

            Ok(update)
        })
    }

    fn get_calibrated_sample(&self) -> AccelerometerResult<Value> {
        self.chip.as_ref().raw_measurement().and_then(|mut value| {
            self.zero.as_ref().map(|zero| {
                value.mut_sub(zero);
            });

            Ok(value)
        })
    }

    #[allow(dead_code)]
    fn estimate_velocity(
        &self,
        new_acceleration_time: SystemTime,
        new_acceleration: Value,
    ) -> Value {
        let previous_measurement = self.previous_measurement.borrow();

        match previous_measurement.estimated_velocity {
            Some(velocity) => {
                let delta_time = new_acceleration_time
                    .duration_since(previous_measurement.time)
                    .unwrap();

                let delta_veclocity = new_acceleration.mul(delta_time.as_secs_f64());

                velocity.add(&delta_veclocity)
            }
            None => Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::{chips::AccelerometerChip, utils::Clock, Accelerometer, Value};
    use std::{
        cell::RefCell,
        time::{Duration, SystemTime},
    };

    const SECONDS_BETWEEN_MOCK_CLOCK_TICKS: f64 = 0.005;

    struct MockAccelerometerHandle {
        measurement_index: RefCell<usize>,
        measurements: Vec<Value>,
    }

    impl MockAccelerometerHandle {
        fn new_from_values<'a, I>(vals: I) -> Self
        where
            I: Iterator<Item = &'a Value>,
        {
            let mut m = Vec::new();

            for v in vals {
                m.push(*v);
            }

            MockAccelerometerHandle {
                measurement_index: RefCell::new(0),
                measurements: m,
            }
        }

        fn new_random(count: u8) -> Self {
            let mut measurements = Vec::new();
            for _ in 0..count {
                measurements.push(Value {
                    x: rand::thread_rng().gen_range(-9.8..9.8),
                    y: rand::thread_rng().gen_range(-9.8..9.8),
                    z: rand::thread_rng().gen_range(-9.8..9.8),
                });
            }

            MockAccelerometerHandle::new_from_values(measurements.iter())
        }
    }

    struct MockClock {
        start: SystemTime,
        calls: RefCell<u32>,
        delay: Duration,
    }
    impl Clock for MockClock {
        fn now(&self) -> std::time::SystemTime {
            let call = self.calls.replace_with(|prev| *prev + 1);
            let delay = Duration::from_secs_f64(self.delay.as_secs_f64() * call as f64);

            self.start.checked_add(delay).unwrap()
        }
    }
    impl Default for MockClock {
        fn default() -> Self {
            MockClock {
                start: SystemTime::now(),
                calls: RefCell::new(0),
                delay: Duration::from_secs_f64(SECONDS_BETWEEN_MOCK_CLOCK_TICKS),
            }
        }
    }

    impl AccelerometerChip for MockAccelerometerHandle {
        fn default_chip_address(&self) -> u8 {
            0
        }

        fn raw_measurement(&self) -> crate::AccelerometerResult<crate::Value> {
            let index = self.measurement_index.replace_with(|prev| *prev + 1);

            Ok(self.measurements[index])
        }

        fn average(&self, sample_count: u8) -> crate::AccelerometerResult<Value> {
            let mut avg: Value = Default::default();

            for _ in 0..sample_count {
                let m = self.raw_measurement().unwrap();

                avg.mut_add(&m);
            }

            avg.mut_div(sample_count as f64);

            Ok(avg)
        }
    }

    #[test]
    fn acceleration() {
        let mock_handle = MockAccelerometerHandle::new_random(1);
        let mock_clock: MockClock = Default::default();
        let expected = mock_handle.measurements[0];

        let uut = Accelerometer::mocked(Box::new(mock_handle), Box::new(mock_clock));

        let actual = uut.measurement().unwrap();

        assert_eq!(actual.acceleration, expected);
    }

    #[test]
    fn acceleration_with_zero() {
        let mock_handle = MockAccelerometerHandle::new_random(6);
        let mock_clock: MockClock = Default::default();
        let raw_measurement = mock_handle.measurements[5];

        let mut uut = Accelerometer::mocked(Box::new(mock_handle), Box::new(mock_clock));

        let zero = uut.auto_set_zero().unwrap();
        let expected = raw_measurement.sub(&zero);

        let actual = uut.measurement().unwrap();

        assert_eq!(actual.acceleration, expected);
    }

    #[test]
    #[ignore]
    fn estimate_velocity() {
        let mock_handle = MockAccelerometerHandle::new_random(2);
        let mock_clock: MockClock = Default::default();
        let acc0 = mock_handle.measurements[0];
        let acc1 = mock_handle.measurements[1];

        let uut = Accelerometer::mocked(Box::new(mock_handle), Box::new(mock_clock));

        let m0 = uut.measurement().unwrap();
        // let expected_velocity = acc0.mul(SECONDS_BETWEEN_MOCK_CLOCK_TICKS);

        assert_eq!(m0.acceleration, acc0);
        // assert_eq!(m0.estimated_velocity, expected_velocity);

        let m1 = uut.measurement().unwrap();
        // let expected_velocity = expected_velocity.add(&acc1.mul(SECONDS_BETWEEN_MOCK_CLOCK_TICKS));

        assert_eq!(m1.acceleration, acc1);
        // assert_eq!(m1.estimated_velocity, expected_velocity);
    }

    #[test]
    fn auto_set_zero() {
        let mock_handle = MockAccelerometerHandle::new_random(5);
        let mock_clock: MockClock = Default::default();
        let avg = Value::average(mock_handle.measurements.iter());

        let mut uut = Accelerometer::mocked(Box::new(mock_handle), Box::new(mock_clock));

        assert_eq!(uut.auto_set_zero().unwrap(), avg);
    }
}
