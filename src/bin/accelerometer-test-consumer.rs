extern crate paho_mqtt as mqtt;
use std::time::Duration;

use accelerometer::{
    mqtt::{QoS, ServiceConfig},
    AccelerometerMeasurement,
};
use clap::Parser;

/// Simple program to interact with an accelerometer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(long, default_value = "/etc/accelerometer.yaml")]
    config_file_path: String,

    /// The maximum number of samples to acquire before exiting
    #[arg(long, default_value = "1")]
    max_samples: u8,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    if args.max_samples == 0 {
        panic!("max_samples must be > 0");
    }

    let config = ServiceConfig::load_from_file(&args.config_file_path, true);

    let client = mqtt::Client::new(config.mqtt_config.url).unwrap();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(config.mqtt_config.user.unwrap())
        .password(config.mqtt_config.password.unwrap())
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail
    client.connect(conn_opts).unwrap();

    client
        .subscribe(
            &config.mqtt_config.measurement_topic,
            QoS::AtMostOnce as i32,
        )
        .unwrap();

    log::info!(
        "Subscribed to topic: {}",
        config.mqtt_config.measurement_topic
    );

    let rx_queue = client.start_consuming();

    let mut sample_count = 0;

    for msg in rx_queue.iter() {
        sample_count = sample_count + 1;

        match msg {
            Some(msg) => {
                let sample: AccelerometerMeasurement =
                    serde_json::from_str(&msg.payload_str()).unwrap();

                log::info!("Received sample {:>5}: {:?}", sample_count, sample);
            }
            None => {}
        }

        if sample_count > args.max_samples {
            break;
        }
    }
}
