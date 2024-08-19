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

    /// A helper function to render a template file from 'html/' directory to String without `Context`
    pub fn render_html_to_string(&self, template_file: &str) -> Result<String, tera::Error> {
        let tera = self.tera();
        let template = format!("html/{template_file}");
        tera.render(&template, &tera::Context::new())
    }

    pub fn tera(&self) -> &Tera {
        self.tera
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn template_man_render_html_to_string_ok() -> Result<()> {
        let templ_man = TemplateManager::init();

        let login_form = templ_man.render_html_to_string("login_form.html")?;
        let login_form_str = include_str!("../templates/html/login_form.html");

        assert_eq!(login_form, login_form_str);

        Ok(())
    }
}
