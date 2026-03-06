fn main() {
    if let Err(e) = sit::app::run() {
        sit::ui::print_error(&e.to_string());
        std::process::exit(1);
    }
}
