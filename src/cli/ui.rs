#![allow(unused)]
use console::{style, Style};
use inquire::ui::{Color, RenderConfig, Styled, Attributes};
use std::fmt::Display;

pub struct UI;

impl UI {
    pub fn logo() {
        let cyan = Style::new().cyan().bold();
        let magenta = Style::new().magenta();
        
        println!(
            "\n{} {} v{}\n",
            cyan.apply_to("⚡ Conduit"),
            magenta.apply_to("CLI"),
            env!("CARGO_PKG_VERSION")
        );
    }

    pub fn render_config() -> RenderConfig<'static> {
        RenderConfig::default().with_prompt_prefix(
            Styled::new("*")
                .with_fg(Color::LightCyan)
                .with_attr(Attributes::BOLD),
        )
    }

    pub fn success<T: Display>(msg: T) {
        println!(
            "{} {}",
            style("✔").green().bold(),
            style(msg).bright()
        );
    }

    pub fn info<T: Display>(msg: T) {
        println!(
            "{} {}",
            style("!").blue().bold(),
            style(msg).dim()
        );
    }

    pub fn warn<T: Display>(msg: T) {
        println!(
            "{} {}",
            style("⚠").yellow().bold(),
            style(msg).yellow()
        );
    }

    pub fn error<T: Display>(msg: T) {
        eprintln!(
            "{} {}",
            style("✘").red().bold(),
            style(msg).red()
        );
    }

    pub fn action<T: Display>(action: &str, target: T) {
        println!(
            "{:>12} {}",
            style(action).magenta().bold(),
            target
        );
    }
}
