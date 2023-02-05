extern crate paho_mqtt as mqtt;
use std::{
    thread,
    time::{self, Duration},
};

use accelerometer::mqtt::{QoS, ServiceConfig};
use clap::Parser;

/// Simple program to interact with an accelerometer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(long, default_value = "/etc/accelerometer.yaml")]
    config_file_path: String,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    // NOTE: We set 'required_username_password=false' even though they are
    //       required... if they're missing, we'd rather panic than prompt
    let config = ServiceConfig::load_from_file(&args.config_file_path, false);

    let client = mqtt::Client::new(config.mqtt_config.url).unwrap();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(config.mqtt_config.user.unwrap())
        .password(config.mqtt_config.password.unwrap())
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail
    client.connect(conn_opts).unwrap();

    let mut acc = accelerometer::Accelerometer::new(&config.device_config);
    acc.auto_set_zero().unwrap();

    let mut sample_count = 0;
    loop {
        let sample = acc.measurement().unwrap();

        let sample_json = serde_json::to_string(&sample).unwrap();

        client
            .publish(mqtt::Message::new(
                &config.mqtt_config.measurement_topic,
                sample_json,
                QoS::AtMostOnce as i32,
            ))
            .unwrap();

        if sample_count % 100 == 0 {
            log::info!("Published sample {:>5}: {:?}", sample_count, sample);
        }

        sample_count = sample_count + 1;

        thread::sleep(time::Duration::from_secs_f64(1.0));
    }
}
