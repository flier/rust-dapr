pub extern crate prost_types;
pub extern crate tonic;

pub mod any;
pub mod client;
pub mod runtime;

pub use dapr_derive::service;

#[cfg(feature = "json")]
#[doc(inline)]
pub use any::json;
#[doc(inline)]
pub use any::{pack, protobuf, Unpack};
#[doc(inline)]
pub use runtime::connect;
