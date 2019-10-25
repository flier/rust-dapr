#![allow(clippy::module_inception)]

//! The Dapr client interface.

tonic::include_proto!("daprclient");

#[tonic::async_trait]
pub trait Events {
    type Error: std::error::Error;

    async fn topic_subscriptions(&self) -> Result<Vec<String>, Self::Error>;

    async fn on_topic_event(&self, event: CloudEventEnvelope) -> Result<(), Self::Error>;
}

#[tonic::async_trait]
pub trait Bindings {
    type Error: std::error::Error;

    async fn bindings_subscriptions(&self) -> Result<Vec<String>, Self::Error>;

    async fn on_binding_event(
        &self,
        event: BindingEventEnvelope,
    ) -> Result<BindingResponseEnvelope, Self::Error>;
}
