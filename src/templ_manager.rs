use core::panic;
use std::sync::OnceLock;

use tera::Tera;
use tracing::info;

#[derive(Debug)]
pub struct TemplateManager {
    tera: &'static Tera,
}

impl TemplateManager {
    pub fn init() -> Self {
        info!(
            "{:<12} - Initializing the Template manager",
            "templ manager"
        );
        static TERA: OnceLock<Tera> = OnceLock::new();
        let tera = TERA.get_or_init(|| {
            Tera::new("templates/**/*").unwrap_or_else(|e| panic!("Parsing error(s): {e}"))
        });
        Self { tera }
    }

    pub fn tera(&self) -> &Tera {
        self.tera
    }
}
