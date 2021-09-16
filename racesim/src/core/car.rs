use crate::core::driver::Driver;
use crate::core::state_handler::StateHandler;
use crate::core::tireset::Tireset;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;

/// * `inlap` - In-lap of the pit stop (0 for tire info at race start)
/// * `tire_start_age` - Age of the tires when they are fitted
/// * `compound` - Compound that is fitted during the pit stop (set empty string for refueling only)
/// * `refuel_mass` - (kg) Fuel mass that is added during the pit stop (set zero for tire-change
/// only)
/// * `driver_initials` - Initials of driver for next stint (set empty string for no driver change)
#[derive(Debug, Deserialize, Clone)]
pub struct StrategyEntry {
    pub inlap: u32,
    pub tire_start_age: u32,
    pub compound: String,
    pub refuel_mass: f64,
    pub driver_initials: String,
}

/// * `car_no` - Car number, e.g. 77
/// * `team` - Team that operates the car, e.g. Mercedes
/// * `manufacturer` - Manufacturer of the car
/// * `color` - Hex-code of the team color (used for plotting)
/// * `t_car` - (s) Time loss per lap due to car abilities
/// * `m_fuel` - (kg) Fuel mass at the race start
/// * `b_fuel_per_lap` - (kg/lap) Fuel consumption per lap
/// * `t_pit_refuel_per_kg` - (s/kg) Standstill time per kg of fuel added in a pit stop
/// * `t_pit_tirechange` - (s) Standstill time to change tires during a pit stop
/// * `t_pit_driverchange` - (s) Standstill time to change drivers during a pit stop
/// * `pit_location` - (m) Location of the pit (must be within the pit lane)
/// * `strategy` - List that contains the strategy entries that determine the pit stops during the
/// race
/// * `p_grid` - Grid position at the race start
#[derive(Debug, Deserialize, Clone)]
pub struct CarPars {
    pub car_no: u32,
    pub team: String,
    pub manufacturer: String,
    pub color: String,
    pub t_car: f64,
    pub m_fuel: f64,
    pub b_fuel_per_lap: f64,
    pub t_pit_refuel_per_kg: Option<f64>,
    pub t_pit_tirechange: f64,
    pub t_pit_driverchange: Option<f64>,
    pub pit_location: f64,
    pub strategy: Vec<StrategyEntry>,
    pub p_grid: u32,
}

#[derive(Debug)]
pub struct Car {
    pub car_no: u32,
    team: String,
    manufacturer: String,
    pub color: String,
    t_car: f64,
    m_fuel: f64,
    b_fuel_per_lap: f64,
    t_pit_refuel_per_kg: Option<f64>,
    t_pit_tirechange: f64,
    t_pit_driverchange: Option<f64>,
    pub pit_location: f64,
    strategy: Vec<StrategyEntry>,
    pub p_grid: u32,
    pub driver: Rc<Driver>,
    pub sh: StateHandler,
    tireset: Tireset,
}

impl Car {
    pub fn new(car_pars: &CarPars, driver: Rc<Driver>) -> Car {
        Car {
            car_no: car_pars.car_no,
            team: car_pars.team.to_owned(),
            manufacturer: car_pars.manufacturer.to_owned(),
            color: car_pars.color.to_owned(),
            t_car: car_pars.t_car,
            m_fuel: car_pars.m_fuel,
            b_fuel_per_lap: car_pars.b_fuel_per_lap,
            t_pit_refuel_per_kg: car_pars.t_pit_refuel_per_kg,
            t_pit_tirechange: car_pars.t_pit_tirechange,
            t_pit_driverchange: car_pars.t_pit_driverchange,
            pit_location: car_pars.pit_location,
            strategy: car_pars.strategy.to_owned(),
            p_grid: car_pars.p_grid,
            driver,
            sh: StateHandler::default(),
            tireset: Tireset::new(
                car_pars.strategy[0].compound.to_owned(),
                car_pars.strategy[0].tire_start_age,
            ),
        }
    }

    /// The method returns the time loss due to car abilities, driver abilities, tire degradation,
    /// and fuel mass.
    pub fn calc_basic_timeloss(&self, s_mass: f64) -> f64 {
        let degr_pars = self.driver.get_degr_pars(&self.tireset.compound);
        self.t_car
            + self.driver.t_driver
            + self.tireset.t_add_tireset(&degr_pars)
            + self.m_fuel * s_mass
    }

    /// The method increases the tire age (for degradation) and reduces the fuel mass (burned during
    /// the lap).
    pub fn drive_lap(&mut self) {
        self.m_fuel -= self.b_fuel_per_lap;

        if self.m_fuel < 0.0 {
            println!(
                "WARNING: Remaining fuel mass of car {} is negative!",
                self.car_no
            );

            // assure that fuel mass is not negative (and therefore lap time decreases)
            self.m_fuel = 0.0;
        }

        self.tireset.drive_lap();
    }

    /// The method determines whether the car enters the pit lane in the current lap according to
    /// its strategy info.
    pub fn pit_this_lap(&self, cur_lap: u32) -> bool {
        self.strategy
            .iter()
            .any(|strat_entry| strat_entry.inlap == cur_lap)
    }

    /// The method checks which of the strategy entries belongs to the current in-lap and returns
    /// it.
    fn get_strategy_entry(&self, inlap: u32) -> StrategyEntry {
        self.strategy
            .iter()
            .find(|&x| x.inlap == inlap)
            .expect("Could not find strategy entry that belongs to the inserted in-lap!")
            .to_owned()
    }

    /// The method checks which of the strategy entries belongs to the current in-lap and performs
    /// the according pit stop in terms of changing tires and refueling (whatever is applicable).
    pub fn perform_pitstop(&mut self, inlap: u32, drivers_list: &HashMap<String, Rc<Driver>>) {
        // get strategy entry
        let strategy_entry = self.get_strategy_entry(inlap);

        // handle tire change
        if !strategy_entry.compound.is_empty() {
            self.tireset = Tireset::new(
                strategy_entry.compound.to_owned(),
                strategy_entry.tire_start_age,
            );
        }

        // handle refueling
        if strategy_entry.refuel_mass > 0.0 {
            self.m_fuel += strategy_entry.refuel_mass
        }

        // handle driver change
        if !strategy_entry.driver_initials.is_empty() {
            self.driver = Rc::clone(
                drivers_list
                    .get(&strategy_entry.driver_initials)
                    .expect("Could not find driver initials in drivers list!"),
            )
        }
    }

    /// The method returns the standstill time during a pit stop.
    pub fn t_add_pit_standstill(&self, inlap: u32) -> f64 {
        // get strategy entry
        let strategy_entry = self.get_strategy_entry(inlap);

        // handle tire change
        let mut t_add_pit_standstill = if !strategy_entry.compound.is_empty() {
            self.t_pit_tirechange
        } else {
            0.0
        };

        // handle refueling
        if strategy_entry.refuel_mass > 0.0 {
            t_add_pit_standstill = t_add_pit_standstill.max(
                strategy_entry.refuel_mass
                    * self
                        .t_pit_refuel_per_kg
                        .expect("Parameter t_pit_refuel_per_kg was not set!"),
            )
        };

        // handle driver change
        if !strategy_entry.driver_initials.is_empty() {
            t_add_pit_standstill = t_add_pit_standstill.max(
                self.t_pit_driverchange
                    .expect("Parameter t_pit_driverchange was not set!"),
            )
        }

        t_add_pit_standstill
    }
}
