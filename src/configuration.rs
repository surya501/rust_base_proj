use std::convert::{TryFrom, TryInto};

use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseConfiguration,
    pub application: AppSettings,
}
#[derive(serde::Deserialize)]
pub struct AppSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfiguration {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            _ => Err(format!("{} is not a valid environment", value)),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to read current directory.");
    let configuration_directory = base_path.join("configuration");

    // Detect if this is a dev environment or production
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse environment var.");

    let base_yaml = config::File::from(
        configuration_directory.join(std::path::Path::new("base").with_extension("yaml")),
    );
    let environment_yaml = config::File::from(
        configuration_directory
            .join(std::path::Path::new(environment.as_str()).with_extension("yaml")),
    );
    let settings = config::Config::builder()
        .add_source(base_yaml)
        .add_source(environment_yaml)
        .build()?;

    settings.try_deserialize::<Settings>()
}

impl DatabaseConfiguration {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
