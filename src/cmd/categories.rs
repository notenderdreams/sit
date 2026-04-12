use colored::Colorize;

use crate::config::Config;
use crate::error::Result;
use crate::print;

pub fn show_categories(cfg: &Config) -> Result<()> {
    let name_width = cfg
        .categories
        .iter()
        .map(|cat| cat.name.chars().count())
        .max()
        .unwrap_or(0)
        + 2;

    print::blank();
    print::header("Commit Categories:");
    print::blank();
    for cat in &cfg.categories {
        let name = format!("{:<name_width$}", cat.name).bold();
        println!("    {}{}", name, cat.description.bright_black());
    }
    print::blank();
    Ok(())
}
