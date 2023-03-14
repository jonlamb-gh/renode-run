use clap::Parser;
use std::path::PathBuf;

// TODO - surface more/all the available config items in opts

/// Run embedded programs in the renode emulator
#[derive(Parser, Debug, Clone, Default)]
#[clap(version)]
pub struct Opts {
    /// Path to renode binary.
    ///
    /// Useful if not on the user's $PATH.
    #[clap(long = "renode", env = "RENODE_RUN_RENODE_BIN")]
    pub renode_bin: Option<PathBuf>,

    /// Path to toml configuration file.
    ///
    /// Defaults to resolving the current cargo workspace's Cargo.toml.
    #[clap(short = 'c', long = "config", env = "RENODE_RUN_CONFIG_FILE")]
    pub config: Option<PathBuf>,

    /// Generate output artifacts in this directory instead of a temporary directory
    #[clap(short = 'o', long = "output", env = "RENODE_RUN_OUTPUT_DIR")]
    pub output_dir: Option<PathBuf>,

    /// Generate, but don't run the Renode script
    #[clap(long)]
    pub no_run: bool,

    /// Input ELF executable
    pub input: PathBuf,
}
