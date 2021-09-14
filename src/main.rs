use rustypaste_cli::args::Args;
use std::process;

pub fn main() {
    let args = Args::parse();
    match rustypaste_cli::run(args) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1)
        }
    }
}
