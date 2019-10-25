//! Convert between `Any` type and primitives types.

use std::any::type_name;
use std::convert::TryInto;

use prost_types::Any;

const RUST_LANG_URL: &str = "rust-lang.org";

/// Pack the given data as a byte array in native byte order.
pub fn pack<T>(value: T) -> Option<Any>
where
    T: AsRef<[u8]>,
{
    Some(Any {
        value: value.as_ref().to_vec(),
        type_url: format!("{}/{}", RUST_LANG_URL, type_name::<T>()),
    })
}

/// A value-to-value conversion that consumes the input value.
pub trait IntoAny {
    /// Performs the conversion.
    fn into_any(self) -> Option<Any>;
}

impl<T> IntoAny for Option<T>
where
    T: IntoAny,
{
    fn into_any(self) -> Option<Any> {
        self.and_then(|value| value.into_any())
    }
}

impl IntoAny for bool {
    fn into_any(self) -> Option<Any> {
        Some(Any {
            value: vec![if self { 1 } else { 0 }],
            type_url: format!("{}/bool", RUST_LANG_URL),
        })
    }
}

impl IntoAny for () {
    fn into_any(self) -> Option<Any> {
        Some(Any {
            value: vec![],
            type_url: format!("{}/()", RUST_LANG_URL),
        })
    }
}

macro_rules! impl_into_any {
    ($ty:ty) => {
        impl IntoAny for $ty {
            fn into_any(self) -> Option<Any> {
                Some(Any {
                    value: self.to_ne_bytes().to_vec(),
                    type_url: format!("{}/{}", RUST_LANG_URL, stringify!($ty)),
                })
            }
        }
    };
}

impl_into_any!(u8);
impl_into_any!(u16);
impl_into_any!(u32);
impl_into_any!(u64);
impl_into_any!(u128);
impl_into_any!(usize);
impl_into_any!(i8);
impl_into_any!(i16);
impl_into_any!(i32);
impl_into_any!(i64);
impl_into_any!(i128);
impl_into_any!(isize);

impl IntoAny for &str {
    fn into_any(self) -> Option<Any> {
        Some(Any {
            value: self.as_bytes().to_vec(),
            type_url: format!("{}/str", RUST_LANG_URL),
        })
    }
}

impl IntoAny for String {
    fn into_any(self) -> Option<Any> {
        Some(Any {
            value: self.as_bytes().to_vec(),
            type_url: format!("{}/String", RUST_LANG_URL),
        })
    }
}

/// Deserialize an instance of type T from a byte array in native byte order.
pub trait Unpack {
    /// Deserialize an instance of type T.
    fn unpack<T>(self) -> Result<T, T::Error>
    where
        T: TryFromAny;
}

impl Unpack for Any {
    fn unpack<T>(self) -> Result<T, T::Error>
    where
        T: TryFromAny,
    {
        T::try_from(self)
    }
}

/// Try convert `Any` to the given type.
pub trait TryFromAny: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Performs the conversion.
    fn try_from(any: Any) -> Result<Self, Self::Error>;
}

macro_rules! impl_try_from_any {
    ($ty:ty) => {
        impl TryFromAny for $ty {
            type Error = std::array::TryFromSliceError;

            fn try_from(any: Any) -> Result<Self, Self::Error> {
                any.value.as_slice().try_into().map(<$ty>::from_ne_bytes)
            }
        }
    };
}

impl_try_from_any!(u8);
impl_try_from_any!(u16);
impl_try_from_any!(u32);
impl_try_from_any!(u64);
impl_try_from_any!(u128);
impl_try_from_any!(usize);
impl_try_from_any!(i8);
impl_try_from_any!(i16);
impl_try_from_any!(i32);
impl_try_from_any!(i64);
impl_try_from_any!(i128);
impl_try_from_any!(isize);

impl TryFromAny for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        String::from_utf8(any.value)
    }
}

/// Serialize the given data structure as a Protobuf message.
pub fn protobuf<T>(value: &T) -> Option<Any>
where
    T: prost::Message,
{
    let mut buf = vec![];

    value.encode(&mut buf).ok()?;

    Some(Any {
        value: buf,
        type_url: format!("{}/{}", RUST_LANG_URL, type_name::<T>()),
    })
}

/// Serialize and Deserialize `Any` type as Protobuf message.
pub mod protobuf {
    use prost_types::Any;

    /// Deserialize an instance of type T from Protobuf message.
    pub fn unpack<T>(any: &Any) -> Result<T, prost::DecodeError>
    where
        T: prost::Message + Default,
    {
        any.unpack()
    }

    /// Deserialize an instance of type T from Protobuf message.
    pub trait Unpack {
        /// Deserialize an instance of type T.
        fn unpack<T>(&self) -> Result<T, prost::DecodeError>
        where
            T: prost::Message + Default;
    }

    impl Unpack for Any {
        fn unpack<T>(&self) -> Result<T, prost::DecodeError>
        where
            T: prost::Message + Default,
        {
            T::decode(&self.value)
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "json")] {
        impl IntoAny for serde_json::Value {
            fn into_any(self) -> Option<Any> {
                serde_json::to_vec(&self).ok().map(|value| Any {
                    value,
                    type_url: format!("{}/JSON", RUST_LANG_URL),
                })
            }
        }

        /// Serialize the given data structure as a JSON text.
        pub fn json<T: ?Sized>(value: &T) -> Option<Any>
        where
            T: serde::Serialize,
        {
            serde_json::to_vec(value).ok().map(|value| Any {
                value,
                type_url: format!("{}/{}", RUST_LANG_URL, type_name::<T>()),
            })
        }

        /// Serialize and Deserialize `Any` type as JSON text.
        pub mod json{
            use prost_types::Any;

            /// Deserialize an instance of type T from JSON text.
            pub fn unpack<'a, T>(any: &'a Any) -> Result<T, serde_json::error::Error>
            where
                T: serde::Deserialize<'a>
            {
                any.unpack()
            }

            /// Deserialize an instance of type T from JSON text.
            pub trait Unpack {
                /// Deserialize an instance of type T.
                fn unpack<'a, T>(&'a self) -> Result<T, serde_json::error::Error>
                where
                    T: serde::Deserialize<'a>;
            }

            impl Unpack for Any {
                fn unpack<'a, T>(&'a self) -> Result<T, serde_json::error::Error>
                where
                    T: serde::Deserialize<'a>,
                {
                    serde_json::from_slice(&self.value)
                }
            }
        }
    }
}
