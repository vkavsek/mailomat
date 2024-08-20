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
            "{:<20} - Initializing the Template manager",
            "templ manager"
        );
        static TERA: OnceLock<Tera> = OnceLock::new();
        let tera = TERA.get_or_init(|| {
            Tera::new("templates/**/*").unwrap_or_else(|e| panic!("Parsing error(s): {e}"))
        });
        Self { tera }
    }

    /// A helper function to render a template file from 'html/' directory to String
    pub fn render_html_to_string(
        &self,
        ctx: &tera::Context,
        template_file: &str,
    ) -> Result<String, tera::Error> {
        let tera = self.tera();
        let template = format!("html/{template_file}");
        tera.render(&template, ctx)
    }

    pub fn tera(&self) -> &Tera {
        self.tera
    }
}
