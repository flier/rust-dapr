//! The Dapr runtime API.

use std::collections::HashMap;
use std::convert::{AsMut, AsRef};

use prost_types::Any;
use tonic::{
    codegen::{Body, HttpBody, StdError},
    Request,
};

use crate::{
    any::IntoAny,
    error::{Error, Result},
};

tonic::include_proto!("dapr");

pub type Metadata = HashMap<String, String>;

/// Dapr runtime API
#[repr(transparent)]
pub struct Runtime<T>(client::DaprClient<T>);

impl<T> AsRef<client::DaprClient<T>> for Runtime<T> {
    fn as_ref(&self) -> &client::DaprClient<T> {
        &self.0
    }
}

impl<T> AsMut<client::DaprClient<T>> for Runtime<T> {
    fn as_mut(&mut self) -> &mut client::DaprClient<T> {
        &mut self.0
    }
}

/// Opens a gRPC connection to a Dapr runtime.
pub fn connect<D>(dst: D) -> Result<Runtime<tonic::transport::Channel>>
where
    D: std::convert::TryInto<tonic::transport::Endpoint>,
    D::Error: Into<StdError>,
{
    client::DaprClient::connect(dst)
        .map(Runtime)
        .map_err(Error::from)
}

impl<T> Runtime<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::ResponseBody: Body + HttpBody + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    <T::ResponseBody as HttpBody>::Data: Into<bytes::Bytes> + Send,
{
    /// Check if the service is ready.
    pub async fn ready(&mut self) -> Result<()> {
        self.0.ready().await.map_err(Error::from)
    }

    /// Invoke a method in a Dapr enabled app.
    pub async fn invoke_service<I, M, D>(
        &mut self,
        app_id: I,
        method_name: M,
        data: D,
    ) -> Result<(Option<Any>, Metadata)>
    where
        I: Into<String>,
        M: Into<String>,
        D: IntoAny,
    {
        self.0
            .invoke_service(Request::new(InvokeServiceEnvelope {
                id: app_id.into(),
                method: method_name.into(),
                data: data.into_any(),
                ..Default::default()
            }))
            .await
            .map(|res| {
                let InvokeServiceResponseEnvelope { data, metadata } = res.into_inner();

                (data, metadata)
            })
            .map_err(Error::from)
    }

    /// Invoke an Dapr output binding.
    pub async fn invoke_binding<S, D>(&mut self, name: S, data: D) -> Result<()>
    where
        S: Into<String>,
        D: IntoAny,
    {
        self.0
            .invoke_binding(Request::new(InvokeBindingEnvelope {
                name: name.into(),
                data: data.into_any(),
                ..Default::default()
            }))
            .await
            .map(|res| res.into_inner())
            .map_err(Error::from)
    }

    /// Publish a payload to multiple consumers who are listening on a topic.
    ///
    /// Dapr guarantees at least once semantics for this endpoint.
    pub async fn publish_event<S, D>(&mut self, topic: S, data: D) -> Result<()>
    where
        S: Into<String>,
        D: IntoAny,
    {
        self.0
            .publish_event(Request::new(PublishEventEnvelope {
                topic: topic.into(),
                data: data.into_any(),
            }))
            .await
            .map(|res| res.into_inner())
            .map_err(Error::from)
    }

    /// Get the state for a specific key.
    pub async fn get_state<S>(&mut self, key: S) -> Result<(Option<Any>, String)>
    where
        S: Into<String>,
    {
        self.0
            .get_state(Request::new(GetStateEnvelope {
                key: key.into(),
                ..Default::default()
            }))
            .await
            .map(|res| {
                let GetStateResponseEnvelope { data, etag } = res.into_inner();

                (data, etag)
            })
            .map_err(Error::from)
    }

    /// Save an array of state objects.
    pub async fn save_state<I, S>(&mut self, requests: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<StateRequest>,
    {
        self.0
            .save_state(Request::new(SaveStateEnvelope {
                requests: requests.into_iter().map(|state| state.into()).collect(),
            }))
            .await
            .map(|res| res.into_inner())
            .map_err(Error::from)
    }

    /// Delete the state for a specific key.
    pub async fn delete_state<S>(&mut self, key: S) -> Result<()>
    where
        S: Into<String>,
    {
        self.0
            .delete_state(Request::new(DeleteStateEnvelope {
                key: key.into(),
                ..Default::default()
            }))
            .await
            .map(|res| res.into_inner())
            .map_err(Error::from)
    }
}

impl<'a, T> From<&'a T> for StateRequest
where
    T: Clone + Into<StateRequest>,
{
    fn from(state: &'a T) -> Self {
        state.clone().into()
    }
}

impl<K, V> From<(K, V)> for StateRequest
where
    K: Into<String>,
    V: IntoAny,
{
    fn from((key, value): (K, V)) -> Self {
        StateRequest {
            key: key.into(),
            value: value.into_any(),
            ..Default::default()
        }
    }
}
