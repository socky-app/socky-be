use serde::Serialize;

/// Error details used during request logging.
#[derive(Serialize, Debug)]
pub struct ErrorDetails {
    pub error_message: String,
    pub error_type: String,
    pub error_debug: String,
    pub error_chain: Vec<ErrorLevel>,
    pub client_message: String,
}

impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.client_message)
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorLevel {
    pub depth: usize,
    pub error_type: String,
    pub message: String,
}

impl ErrorDetails {
    pub fn new(err: &dyn std::error::Error, client_message: String) -> Self {
        let error_chain = extract_error_chain(err);

        // Build the dotted path by mapping over the extracted types
        let error_type_path = error_chain
            .iter()
            .map(|level| level.error_type.as_str())
            .collect::<Vec<&str>>()
            .join(".");

        Self {
            error_message: err.to_string(),
            error_type: error_type_path,
            error_debug: format!("{:?}", err),
            error_chain,
            client_message,
        }
    }
}

/// Walks the error chain and builds the JSON-friendly array
fn extract_error_chain(err: &dyn std::error::Error) -> Vec<ErrorLevel> {
    let mut chain = Vec::new();
    let mut current_err = Some(err);
    let mut depth = 0;

    while let Some(e) = current_err {
        chain.push(ErrorLevel {
            depth,
            error_type: extract_type_name(e),
            message: e.to_string(),
        });

        current_err = e.source();
        depth += 1;
    }

    chain
}

/// Extracts the enum variant or struct name from any error
fn extract_type_name(err: &dyn std::error::Error) -> String {
    let debug_str = format!("{:?}", err);
    let end_index = debug_str
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(debug_str.len());

    debug_str[..end_index].to_string()
}
