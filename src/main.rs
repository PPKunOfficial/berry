#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    berry_api_api::start_server().await?;
    Ok(())
}
