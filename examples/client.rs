use dapr::client::server::DaprClientServer;

/// MyServer is our user app
pub struct MyServer {}

#[dapr::service]
pub trait MyService: dapr::client::Events + dapr::client::Bindings {
    fn my_method(&self, name: String) -> String;
}

impl MyService for MyServer {
    /// Sample method to invoke
    fn my_method(&self, name: String) -> String {
        format!("Hi there, {}!", name)
    }
}

#[tonic::async_trait]
impl dapr::client::Events for MyServer {
    type Error = tonic::Status;

    async fn topic_subscriptions(&self) -> Result<Vec<String>, Self::Error> {
        Ok(vec!["TopicA".to_owned()])
    }

    async fn on_topic_event(
        &self,
        event: dapr::client::CloudEventEnvelope,
    ) -> Result<(), Self::Error> {
        println!("Topic message arrived: {:?}", event);

        Ok(())
    }
}

#[tonic::async_trait]
impl dapr::client::Bindings for MyServer {
    type Error = tonic::Status;

    async fn bindings_subscriptions(&self) -> Result<Vec<String>, Self::Error> {
        Ok(vec!["TopicA".to_owned()])
    }

    async fn on_binding_event(
        &self,
        event: dapr::client::BindingEventEnvelope,
    ) -> Result<dapr::client::BindingResponseEnvelope, Self::Error> {
        println!("Invoked from binding: {:?}", event);

        Ok(Default::default())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:4000".parse().unwrap();
    let server = MyServiceClient(MyServer {});

    // create grpc server
    tonic::transport::Server::builder()
        .serve(addr, DaprClientServer::new(server))
        .await?;

    Ok(())
}
