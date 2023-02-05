use std::{fs, thread, time};

use accelerometer::{Accelerometer, DeviceConfig};
use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    device_config: DeviceConfig,
}

/// Simple program to interact with an accelerometer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(long, default_value = "/etc/accelerometer.yaml")]
    config_file_path: String,

    /// Use the accelerometer's current values to calibrate an effective "zero"
    #[arg(long)]
    no_zero: bool,

    /// The amount of time (in seconds) between each sample measurement
    #[arg(long, default_value = "0.1")]
    sample_duration: f64,

    /// The maximum number of samples to acquire before exiting
    #[arg(long, default_value = "1")]
    max_samples: u8,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    if args.sample_duration <= 0.0 {
        panic!("sample_duration must be > 0.0");
    }

    if args.max_samples == 0 {
        panic!("max_samples must be > 0");
    }

    let config = fs::read_to_string(args.config_file_path).unwrap();
    let config: Config = serde_yaml::from_str(&config).unwrap();

    let mut acc = Accelerometer::new(&config.device_config);

    if !args.no_zero {
        acc.auto_set_zero().unwrap();
    }

    let mut sample_count = 0;
    while sample_count < args.max_samples {
        log::info!(
            "Sample {:>5}: {:?}",
            sample_count,
            acc.measurement().unwrap()
        );
        sample_count = sample_count + 1;

        if sample_count < args.max_samples {
            thread::sleep(time::Duration::from_secs_f64(args.sample_duration));
        }
    }
}
