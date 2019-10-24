use prost_types::Any;
use tonic::{transport::Server, Request, Response, Status};

use dapr::client::{
    server::{DaprClient, DaprClientServer},
    BindingEventEnvelope, BindingResponseEnvelope, CloudEventEnvelope,
    GetBindingsSubscriptionsEnvelope, GetTopicSubscriptionsEnvelope, InvokeEnvelope,
};

/// MyServer is our user app
pub struct MyServer {}

impl MyServer {
    /// Sample method to invoke
    fn my_method(&self) -> &str {
        "Hi there!"
    }
}

#[tonic::async_trait]
impl DaprClient for MyServer {
    /// This method gets invoked when a remote service has called the app through Dapr
    /// The payload carries a Method to identify the method, a set of metadata properties and an optional payload
    async fn on_invoke(&self, request: Request<InvokeEnvelope>) -> Result<Response<Any>, Status> {
        let data = request.get_ref().data.as_ref();

        println!("Got invoked with {:?}", data);

        match request.get_ref().method.as_str() {
            "MyMethod" => {
                let response = self.my_method();

                Ok(Response::new(Any {
                    value: response.as_bytes().to_vec(),
                    ..Default::default()
                }))
            }
            _ => Err(tonic::Status::unimplemented("Not yet implemented")),
        }
    }

    /// Dapr will call this method to get the list of topics the app wants to subscribe to.
    /// In this example, we are telling Dapr To subscribe to a topic named TopicA
    async fn get_topic_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetTopicSubscriptionsEnvelope>, Status> {
        println!("Invoked from topic subscriptions");

        Ok(Response::new(GetTopicSubscriptionsEnvelope {
            topics: vec!["TopicA".to_owned()],
        }))
    }

    /// Dapper will call this method to get the list of bindings the app will get invoked by.
    /// In this example, we are telling Dapr To invoke our app with a binding named storage
    async fn get_bindings_subscriptions(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetBindingsSubscriptionsEnvelope>, Status> {
        println!("Invoked from bindings subscriptions");

        Ok(Response::new(GetBindingsSubscriptionsEnvelope {
            bindings: vec!["storage".to_owned()],
        }))
    }

    /// This method gets invoked every time a new event is fired from a registerd binding.
    /// The message carries the binding name, a payload and optional metadata
    async fn on_binding_event(
        &self,
        request: Request<BindingEventEnvelope>,
    ) -> Result<Response<BindingResponseEnvelope>, Status> {
        println!("Invoked from binding: {:?}", request);

        Ok(Response::new(BindingResponseEnvelope::default()))
    }

    /// This method is fired whenever a message has been published to a topic that has been subscribed.
    /// Dapr sends published messages in a CloudEvents 0.3 envelope.
    async fn on_topic_event(
        &self,
        request: Request<CloudEventEnvelope>,
    ) -> Result<Response<()>, Status> {
        println!("Topic message arrived: {:?}", request);

        Ok(Response::new(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:4000".parse().unwrap();
    let server = MyServer {};

    // create grpc server
    Server::builder()
        .serve(addr, DaprClientServer::new(server))
        .await?;

    Ok(())
}
