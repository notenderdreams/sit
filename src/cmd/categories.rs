use colored::Colorize;

use crate::config::Config;
use crate::error::Result;
use crate::print;

pub fn show_categories(cfg: &Config) -> Result<()> {
    print::blank();
    print::header("Commit Categories:");
    print::blank();
    for cat in &cfg.categories {
        println!(
            "    {}  {}",
            cat.name.bold(),
            cat.description.bright_black()
        );
    }
    print::blank();
    Ok(())
}
