/// Error details used during request logging.
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_data: String,
    pub client_message: String,
}

impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.client_message)
    }
}