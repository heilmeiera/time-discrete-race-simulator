use clap::{AppSettings, Clap};
use std::path::PathBuf;

#[derive(Debug, Clap, Clone)]
#[clap(
    version = "0.1.0",
    author = "Alexander Heilmeier <alexander.heilmeier@tum.de>",
    name = "RS-TD",
    about = "A time-discrete race simulator written in Rust"
)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct SimOpts {
    // FLAGS ---------------------------------------------------------------------------------------
    /// Activate debug printing (not usable in case GUI/real-time simulation is activated)
    #[clap(short, long, conflicts_with = "gui")]
    pub debug: bool,

    /// Activate GUI (race is then simulated in real-time with the inserted real-time factor)
    #[clap(short, long, conflicts_with = "debug")]
    pub gui: bool,

    // OPTIONS -------------------------------------------------------------------------------------
    /// Set number of simulation runs (must be one in case GUI/real-time simulation is activated)
    #[clap(short, long, default_value = "1")]
    pub no_sim_runs: u32,

    /// Set path to the simulation parameter file
    #[clap(parse(from_os_str), short, long)]
    pub parfile_path: PathBuf,

    /// Set real-time factor (only relevant in case GUI/real-time simulation is activated)
    #[clap(short, long, default_value = "1.0")]
    pub realtime_factor: f64,

    /// Set simulation timestep size in seconds, should be in the range [0.001, 1.0]
    #[clap(short, long, default_value = "0.2")]
    pub timestep_size: f64,
}
