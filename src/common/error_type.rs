/// Defines how an error should be formatted as a dot-separated path for logging.
pub trait ErrorType: AsRef<str> {
    fn error_type(&self) -> String {
        self.as_ref().to_string()
    }
}

/// A macro to quickly implement `ErrorType` for errors that purely wrap other errors.
#[macro_export]
macro_rules! impl_error_type {
    (
        $enum_name:ident {
            delegate: [ $( $delegate:ident ),* $(,)? ],
            // Matches the ident, plus an optional block of parens or braces
            terminal: [ $( $terminal:ident $( $rest:tt )? ),* $(,)? ]
            $(,)?
        }
    ) => {
        impl ErrorType for $enum_name {
            fn error_type(&self) -> String {
                let prefix = self.as_ref();
                match self {
                    $(
                        Self::$delegate(e) => format!("{}.{}", prefix, e.error_type()),
                    )*
                    $(
                        // Inserts the ident and the destructuring syntax (if any)
                        Self::$terminal $($rest)? => prefix.to_string(),
                    )*
                }
            }
        }
    };
}
