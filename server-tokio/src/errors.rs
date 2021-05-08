use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    /// An error message should be returned to the client,
    /// but the server should continue as usual.
    /// Typically used in response to client errors.
    /// Could also be used if there was an error server-side
    /// but the connection should not be affected
    /// (e.g. an io error on a single file).
    #[error("Non Fatal Error: {msg}")]
    NonFatal {
        msg: String,
        #[source]
        cause: Option<Box<dyn std::error::Error>>,
    },
    /// The error message will be converted to an Rerror,
    /// sent, and then the server will shut down.
    #[error("Notified Fatal Error")]
    NotifiedFatal(#[source] Box<dyn std::error::Error>),
    /// The server will immediately shut down.
    /// Nothing is sent to the client.
    #[error("Immediate Fatal Error")]
    ImmediateFatal(#[source] Box<dyn std::error::Error>),
}


pub type NineResult<T> = Result<T, ServerError>;

pub fn rerr<T, S: Into<String>>(s: S) -> NineResult<T> {
    Err(ServerError::NonFatal {
        msg: s.into(),
        cause: None,
    })
}