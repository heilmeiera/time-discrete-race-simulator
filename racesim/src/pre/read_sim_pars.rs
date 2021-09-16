use crate::core::car::CarPars;
use crate::core::driver::DriverPars;
use crate::core::race::RacePars;
use crate::core::track::TrackPars;
use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::Path;

/// SimPars is used to store all other parameter structs.
#[derive(Debug, Deserialize, Clone)]
pub struct SimPars {
    pub race_pars: RacePars,
    pub track_pars: TrackPars,
    pub driver_pars_all: HashMap<String, DriverPars>,
    pub car_pars_all: HashMap<u32, CarPars>,
}

/// read_sim_pars reads the JSON file and decodes the JSON string into the simulation parameters
/// struct.
pub fn read_sim_pars(filepath: &Path) -> anyhow::Result<SimPars> {
    // open file
    let fh = OpenOptions::new()
        .read(true)
        .open(filepath)
        .context(format!(
            "Failed to open parameter file {}!",
            filepath.to_str().unwrap()
        ))?;

    // read and parse parameter file content
    let pars = serde_json::from_reader(&fh).context(format!(
        "Failed to parse parameter file {}!",
        filepath.to_str().unwrap()
    ))?;
    Ok(pars)
}
