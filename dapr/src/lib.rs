//! Dapr is a portable, event-driven, serverless runtime for building distributed applications across cloud and edge.
//!
//! This is the Dapr SDK for Rust, based on the auto-generated protobuf client.

#[doc(hidden)]
pub extern crate bytes;
#[doc(hidden)]
pub extern crate prost_types;
#[doc(hidden)]
pub extern crate serde;
#[doc(hidden)]
pub extern crate tonic;

#[cfg(feature = "mocking")]
#[macro_use]
pub extern crate simulacrum;

#[doc(hidden)]
pub use async_trait::async_trait;
pub use dapr_derive::{service, stub};

pub mod any;
pub mod client;
mod error;
pub mod runtime;

pub use error::Error;

#[cfg(feature = "json")]
#[doc(inline)]
pub use any::json;
#[doc(inline)]
pub use any::{pack, protobuf, Unpack};
#[doc(inline)]
pub use runtime::connect;
