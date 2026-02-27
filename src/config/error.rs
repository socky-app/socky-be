use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid configuration: {}", details.join("; "))]
pub struct ConfigError {
    pub details: Vec<String>,
}

impl From<figment::Error> for ConfigError {
    fn from(value: figment::Error) -> Self {
        let details = value
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>();

            ConfigError { details }
    }
}
