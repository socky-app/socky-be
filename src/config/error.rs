use std::fmt;

use thiserror::Error;

// TODO: Fix Display
#[derive(Debug, Error)]
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

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n====================================================")?;
        writeln!(f, "❌ CONFIGURATION ERROR")?;
        writeln!(f, "====================================================")?;

        for (i, msg) in self.details.iter().enumerate() {
            writeln!(f, "{}. Issue: {}", i + 1, msg)?;
        }

        writeln!(f, "----------------------------------------------------")?;
        writeln!(f, "💡 TROUBLESHOOTING:")?;
        writeln!(f, "Ensure these Environment Variables are set (double __ for nesting):")?;
        writeln!(f, "   - SOCKY_DB__USER")?;
        writeln!(f, "   - SOCKY_DB__PASSWORD")?;
        writeln!(f, "   - SOCKY_DB__HOST")?;
        writeln!(f, "   - SOCKY_AUTH__PASSWORD_PEPPER")?;
        writeln!(f, "   - SOCKY_AUTH__ACCESS_TOKEN_SECRET")?;
        writeln!(f, "   - SOCKY_AUTH__REFRESH_TOKEN_SECRET")?;
        writeln!(f, "   - SOCKY_AUTH__REFRESH_TOKEN_PEPPER")?;
        writeln!(f, "====================================================")
    }
}
