use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time() // For early dev
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {addr}");

    mailer::serve(listener)?.await?;

    Ok(())
}
