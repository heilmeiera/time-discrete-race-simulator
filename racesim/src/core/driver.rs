use crate::core::tireset::DegrPars;
use serde::Deserialize;
use std::collections::HashMap;

/// * `initials` - Driver initials, e.g. BOT
/// * `name` - Driver name, e.g. Valtteri Bottas
/// * `t_driver` - (s) Time loss per lap due to driver abilities
/// * `t_teamorder` - (s) Team order time delta (negative or positive)
/// * `vel_max` - (km/h) Maximum velocity during qualifying
/// * `degr_pars_all` - Map containing the degradation parameters for all relevant tire compounds
#[derive(Debug, Deserialize, Clone)]
pub struct DriverPars {
    pub initials: String,
    pub name: String,
    pub t_driver: f64,
    pub t_teamorder: f64,
    pub vel_max: f64,
    pub degr_pars_all: HashMap<String, DegrPars>,
}

#[derive(Debug)]
pub struct Driver {
    pub initials: String,
    name: String,
    pub t_driver: f64,
    t_teamorder: f64,
    vel_max: f64,
    degr_pars_all: HashMap<String, DegrPars>,
}

impl Driver {
    pub fn new(driver_pars: &DriverPars) -> Driver {
        Driver {
            initials: driver_pars.initials.to_owned(),
            name: driver_pars.name.to_owned(),
            t_driver: driver_pars.t_driver,
            t_teamorder: driver_pars.t_teamorder,
            vel_max: driver_pars.vel_max,
            degr_pars_all: driver_pars.degr_pars_all.to_owned(),
        }
    }

    /// The method returns the degradation parameters of the current driver for the given compound.
    pub fn get_degr_pars(&self, compound: &str) -> DegrPars {
        self.degr_pars_all
            .get(compound)
            .expect("Degradation parameters are not available for the given compound!")
            .to_owned()
    }
}
