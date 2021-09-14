use getopts::Options;
use std::env;
use std::path::PathBuf;
use std::process;

/// Command-line arguments to parse.
#[derive(Debug, Default)]
pub struct Args {
    /// Configuration file.
    pub config: Option<PathBuf>,
    /// Server address.
    pub server: Option<String>,
    /// Authentication token.
    pub auth: Option<String>,
    /// Files to upload.
    pub files: Vec<String>,
}

impl Args {
    /// Parses the command-line arguments.
    pub fn parse() -> Self {
        let mut opts = Options::new();
        opts.optflag("h", "help", "prints help information");
        opts.optflag("v", "version", "prints version information");
        opts.optopt("c", "config", "sets the configuration file", "CONFIG");
        opts.optopt(
            "s",
            "server",
            "sets the address of the rustypaste server",
            "SERVER",
        );
        opts.optopt("a", "auth", "sets the authentication token", "TOKEN");

        let env_args: Vec<String> = env::args().collect();
        let matches = match opts.parse(&env_args[1..]) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Argument error: `{}`", e);
                process::exit(1);
            }
        };

        if matches.opt_present("h") || matches.free.is_empty() {
            let usage = format!(
                "\n{} {} \u{2014} {}.\
                \n\u{221F} written by {}\
                \n\u{221F} licensed under MIT <{}>\
                \n\nUsage:\n    {} [options] <file(s)>",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_DESCRIPTION"),
                env!("CARGO_PKG_AUTHORS"),
                env!("CARGO_PKG_REPOSITORY"),
                env!("CARGO_PKG_NAME"),
            );
            println!("{}", opts.usage(&usage));
            process::exit(0)
        }

        if matches.opt_present("v") {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            process::exit(0)
        }

        Args {
            config: matches.opt_str("c").map(PathBuf::from),
            server: matches.opt_str("s"),
            auth: matches.opt_str("a"),
            files: matches.free,
        }
    }
}
