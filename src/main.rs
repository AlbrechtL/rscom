fn main() {
    if let Err(err) = rscom::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
