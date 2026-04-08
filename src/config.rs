use clap::Parser;

const VERSION: &str = "0.1.0";

#[derive(Debug, Parser)]
#[clap(version = VERSION)]
struct Args {
    /// set AF_INET listen address
    #[clap(long, default_value = "localhost")]
    address: String,

    /// set socket address family: "inet" for AF_INET, "unix" for AF_UNIX
    #[clap(long, default_value = "inet")]
    af: String,

    /// set working directory
    #[clap(long, default_value = "")]
    dir: String,

    /// disable message history
    #[clap(long)]
    disable_history: bool,

    /// enable filtering of own messages
    #[clap(long)]
    filter_own: bool,

    /// set logging level
    #[clap(long, env = "RUST_LOG", default_value = "warn,nuqql_matrixd_rs=info")]
    loglevel: String,

    /// set AF_INET listen port
    #[clap(long, default_value_t = 32000)]
    port: u16,

    /// push accounts to client
    #[clap(long)]
    push_accounts: bool,

    /// set AF_UNIX socket file in working directory
    #[clap(long, default_value = "nuqql-matrix.sock")]
    sockfile: String,
}

pub struct Config {
    pub loglevel: String,
}

impl Config {
    pub fn parse() -> Self {
        // parse command line arguments
        let args = Args::parse();

        // create config
        Self {
            loglevel: args.loglevel,
        }
    }
}
