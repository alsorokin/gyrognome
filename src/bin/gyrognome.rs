fn main() {
    if let Err(error) = gyrognome::cli::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
