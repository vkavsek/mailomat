use mailomat::{config::get_or_init_config, App, Result};

#[tokio::main]
async fn main() -> Result<()> {
    mailomat::init_tracing();

    // Blocking here probably doesn't matter since we only have the main thread.
    let config = get_or_init_config();

    let app = App::build_from_config(config.to_owned()).await?;
    mailomat::serve(app).await?;

    Ok(())
}
