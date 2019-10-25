use dapr::Unpack;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the Dapr port and create a connection
    let port: u16 = std::env::var("DAPR_GRPC_PORT")?.parse()?;
    let addr = format!("https://127.0.0.1:{}", port);

    // Create the client
    let mut client = dapr::connect(addr)?;

    // Invoke a method called MyMethod on another Dapr enabled service with id client
    let (res, _) = client
        .invoke_service("client", "my_method", json!({"name": "world"}))
        .await?;

    println!("{:?}", res.unwrap().unpack::<String>()?);

    // Publish a message to the topic TopicA
    client.publish_event("TopicA", "Hi from Pub Sub").await?;

    println!("Published message!");

    // Save state with the key myKey
    client.save_state(&[("myKey", "My State")]).await?;

    println!("Saved state!");

    // Get state for key myKey
    let (res, _) = client.get_state("myKey").await?;

    println!("Got state: {}", res.unwrap().unpack::<String>()?);

    // Delete state for key myKey
    client.delete_state("myKey").await?;

    println!("Got deleted",);

    // Invoke output binding named storage. Make sure you set up a Dapr binding, otherwise this will fail
    client.invoke_binding("storage", "some data").await?;

    println!("Binding invoked");

    Ok(())
}
