use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

/// Dapr error
#[derive(Error, Debug)]
pub enum Error {
    /// gRPC transport error
    #[error("transport error")]
    Transport(#[from] tonic::transport::Error),

    /// gRPC status
    #[error("gRPC status")]
    Grpc(#[from] tonic::Status),

    /// JSON error
    #[error("JSON error")]
    Json(#[from] serde_json::error::Error),
}
