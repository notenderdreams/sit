mod categories;
mod git;
mod picker;
mod ui;

fn interactive_commit() -> Result<(), Box<dyn std::error::Error>> {
    let changes = git::get_status()?;

    if changes.is_empty() {
        println!("\n  \x1b[90mNo changes to commit\x1b[0m\n");
        return Ok(());
    }

    let selected_files = picker::pick_files(changes)?;

    if selected_files.is_empty() {
        println!("  \x1b[90mNo files selected\x1b[0m\n");
        return Ok(());
    }

    let category = ui::select_category()?;
    let message = ui::prompt_message(category)?;
    let description = ui::prompt_description()?;

    let full_message = if category == "none" {
        if description.trim().is_empty() {
            message.clone()
        } else {
            format!("{}\n\n{}", message, description)
        }
    } else if description.trim().is_empty() {
        format!("{}: {}", category, message)
    } else {
        format!("{}: {}\n\n{}", category, message, description)
    };

    git::stage_files(&selected_files)?;
    git::commit(&full_message)?;
    ui::print_success(&full_message);

    Ok(())
}

fn show_help() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n  \x1b[1mCommit Categories:\x1b[0m\n");
    for cat in categories::CATEGORIES {
        println!(
            "    {}  \x1b[1m{}\x1b[0m  \x1b[90m{}\x1b[0m",
            cat.emoji, cat.name, cat.description
        );
    }
    println!();
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let result = if args.len() > 1 {
        match args[1].as_str() {
            "help" | "-h" | "--help" => show_help(),
            "commit" | "c" => interactive_commit(),
            _ => {
                println!("\n  \x1b[1msit\x1b[0m — structured commits\n");
                println!("  \x1b[90mUsage:\x1b[0m");
                println!("    sit          interactive commit");
                println!("    sit help     show categories");
                println!();
                Ok(())
            }
        }
    } else {
        interactive_commit()
    };

    if let Err(e) = result {
        ui::print_error(&e.to_string());
        std::process::exit(1);
    }
}

