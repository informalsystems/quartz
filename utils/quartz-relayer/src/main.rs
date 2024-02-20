use quartz_proto::quartz::{core_client::CoreClient, SessionCreateRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CoreClient::connect("http://localhost:11090").await?;

    let request = tonic::Request::new(SessionCreateRequest {});

    let response = client.session_create(request).await?;
    println!("{:?}", response.into_inner());

    Ok(())
}
