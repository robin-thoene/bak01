use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    load_balancer::run().await
}
