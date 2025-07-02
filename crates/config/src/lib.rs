use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub network: InterfaceConfig,
    pub database_path: String,
}
enum ConfigTypes {
    Toml,
    Json,
    None,
}

fn find_config_type(file_name: &str) -> ConfigTypes {
    let json_file = format!("{}.json", file_name);
    let toml_file = format!("{}.toml", file_name);

    let json_path = Path::new(&json_file);
    let toml_path = Path::new(&toml_file);

    if json_path.exists() {
        ConfigTypes::Json
    } else if toml_path.exists() {
        ConfigTypes::Toml
    } else {
        ConfigTypes::None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterfaceConfig {
    pub interface: String,
    pub port: u16,
}

impl Default for InterfaceConfig {
    fn default() -> Self {
        Self {
            interface: "0.0.0.0".to_string(),
            port: 3250,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: InterfaceConfig::default(),
            database_path: "database.db".to_string(),
        }
    }
}

pub fn load_config(path: &str) -> Config {
    match find_config_type(path) {
        ConfigTypes::Json => {
            let file_path = format!("{}.json", path);
            let file_content = fs::read_to_string(&file_path).expect("Failed to read config file");
            serde_json::from_str(&file_content).expect("Failed to parse config file")
        }
        ConfigTypes::Toml => {
            let file_path = format!("{}.toml", path);
            let file_content = fs::read_to_string(&file_path).expect("Failed to read config file");
            toml::from_str(&file_content).expect("Failed to parse config file")
        }
        ConfigTypes::None => {
            let default_config = Config::default();
            let file_path = format!("{}.json", path);
            let dir = Path::new(&file_path).parent().unwrap();
            fs::create_dir_all(dir).expect("Failed to create directory structure");
            let choice = utils::input::choice(
                "jt",
                false,
                Some("No config file found, create a new one? [j]son/[t]oml: "),
            );
            match choice {
                'j' | 'J' => {
                    let json_content = serde_json::to_string_pretty(&default_config)
                        .expect("Failed to serialize default config to JSON");
                    fs::write(&file_path, json_content)
                        .expect("Failed to write default config file");
                    default_config
                }
                't' | 'T' => {
                    let toml_file_path = format!("{}.toml", path);
                    let toml_content = toml::to_string_pretty(&default_config)
                        .expect("Failed to serialize default config to TOML");
                    fs::write(&toml_file_path, toml_content)
                        .expect("Failed to write default config file");
                    default_config
                }
                _ => panic!("How did you get here?"),
            }
        }
    }
}

pub fn save_config(path: &str, config: &Config) {
    match find_config_type(path) {
        ConfigTypes::Json => {
            let file_path = format!("{}.json", path);
            let json_content =
                serde_json::to_string_pretty(config).expect("Failed to serialize config to JSON");
            fs::write(&file_path, json_content).expect("Failed to write config file");
        }
        ConfigTypes::Toml => {
            let file_path = format!("{}.toml", path);
            let toml_content =
                toml::to_string_pretty(config).expect("Failed to serialize config to TOML");
            fs::write(&file_path, toml_content).expect("Failed to write config file");
        }
        ConfigTypes::None => panic!("No configuration type found"),
    }
}
