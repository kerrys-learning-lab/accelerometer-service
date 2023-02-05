use std::fs;

use dialoguer::{Input, Password};
use serde::Deserialize;

use crate::DeviceConfig;

#[derive(Debug, Deserialize)]
pub struct MqttBrokerConfig {
    pub url: String,
    pub measurement_topic: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceConfig {
    pub mqtt_config: MqttBrokerConfig,

    pub device_config: DeviceConfig,
}

#[derive(Debug)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl ServiceConfig {
    pub fn load_from_file(path: &String, require_username_password: bool) -> ServiceConfig {
        let config = fs::read_to_string(path).unwrap();
        let mut config: ServiceConfig = serde_yaml::from_str(&config).unwrap();

        if require_username_password {
            config.mqtt_config.user.get_or_insert_with(|| {
                Input::new()
                    .with_prompt("MQTT username: ")
                    .interact_text()
                    .unwrap()
            });

            config.mqtt_config.password.get_or_insert_with(|| {
                Password::new()
                    .with_prompt("MQTT password: ")
                    .interact()
                    .unwrap()
            });
        }
        config
    }
}
