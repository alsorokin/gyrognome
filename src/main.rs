fn main() {
    if let Err(error) = gyrognome::cli::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
