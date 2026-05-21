fn main() {
    if let Err(err) = soma_zero::cli::run() {
        eprintln!("soma-zero: {err}");
        std::process::exit(1);
    }
}
