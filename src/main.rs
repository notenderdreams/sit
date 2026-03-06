fn main() {
    if let Err(e) = sit::app::run() {
        sit::print::blank();
        sit::print::error(&e.to_string());
        sit::print::blank();
        std::process::exit(1);
    }
}
