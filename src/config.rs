use clap::Parser;
use std::path::PathBuf;

const DIR_PERMISSIONS: &str = "700";
const ACCOUNTS_FILE: &str = "accounts.json";
const FILE_PERMISSIONS: &str = "600";

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

    /// set permissions of working directory in octal representation
    #[clap(long, value_parser = parse_permissions, default_value = DIR_PERMISSIONS)]
    dir_permissions: u32,

    /// disable message history
    #[clap(long)]
    disable_history: bool,

    /// set permissions of files in octal representation
    #[clap(long, value_parser = parse_permissions, default_value = FILE_PERMISSIONS)]
    file_permissions: u32,

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

fn parse_permissions(string: &str) -> anyhow::Result<u32> {
    let perm = u32::from_str_radix(string, 8)?;
    if perm < 0o100 || perm > 0o777 {
        anyhow::bail!("Invalid permissions: {perm:o}");
    }
    Ok(perm)
}

#[derive(Clone)]
pub struct Config {
    pub dir: PathBuf,
    pub dir_permissions: u32,
    pub accounts_file: PathBuf,
    pub accounts_file_permissions: u32,
    pub session_file_permissions: u32,
    pub db_file_permissions: u32,
    pub loglevel: String,
}

impl Config {
    pub fn parse() -> Self {
        // parse command line arguments
        let args = Args::parse();

        // get directory
        let dir = if args.dir.is_empty() {
            if let Some(mut dir) = dirs::config_dir() {
                dir.push("nuqql-matrixd-rs");
                dir
            } else {
                PathBuf::new()
            }
        } else {
            PathBuf::from(args.dir)
        };

        // get accounts file
        let accounts_file = dir.join(ACCOUNTS_FILE);

        // create config
        Self {
            dir,
            dir_permissions: args.dir_permissions,
            accounts_file,
            accounts_file_permissions: args.file_permissions,
            session_file_permissions: args.file_permissions,
            db_file_permissions: args.file_permissions,
            loglevel: args.loglevel,
        }
    }
}
