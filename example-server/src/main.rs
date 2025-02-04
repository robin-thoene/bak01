use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    example_server::run().await
}
