use std::{env, fs};

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
    pub fn load_from_file(path: &String, prompt_allowed: bool) -> ServiceConfig {
        let config = fs::read_to_string(path).unwrap();
        let mut config: ServiceConfig = serde_yaml::from_str(&config).unwrap();

        config.mqtt_config.user.get_or_insert_with(|| {
            env::var("MQTT_USERNAME").unwrap_or_else(|_| {
                if prompt_allowed {
                    Input::new()
                        .with_prompt("MQTT username: ")
                        .interact_text()
                        .unwrap()
                } else {
                    panic!("No configuration value for mqtt_config.user and MQTT_USERNAME not present in environment")
                }
            })
        });

        config.mqtt_config.password.get_or_insert_with(|| {
            env::var("MQTT_PASSWORD").unwrap_or_else(|_| {
                if prompt_allowed {
                    Password::new()
                        .with_prompt("MQTT password: ")
                        .interact()
                        .unwrap()
                } else {
                    panic!("No configuration value for mqtt_config.password and MQTT_PASSWORD not present in environment")
                }
            })
        });

        config
    }
}
