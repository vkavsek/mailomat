use mailomat::{config::get_or_init_config, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // We have a different logging mechanism for production
    #[cfg(not(debug_assertions))]
    {
        mailomat::init_production_tracing()
    }
    #[cfg(debug_assertions)]
    {
        mailomat::init_dbg_tracing();
    }

    // Blocking here probably doesn't matter since we only have the main thread.
    let config = get_or_init_config();

    let app = mailomat::build(config).await?;
    mailomat::serve(app).await?;

    Ok(())
}
