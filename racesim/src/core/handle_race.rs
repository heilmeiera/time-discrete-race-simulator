use crate::core::race::Race;
use crate::interfaces::gui_interface::{CarState, RaceState, RgbColor, MAX_GUI_UPDATE_FREQUENCY};
use crate::post::race_result::RaceResult;
use crate::pre::read_sim_pars::SimPars;
use anyhow::Context;
use css_color_parser;
use flume::Sender;
use std::thread::sleep;
use std::time::{Duration, Instant};

/// handle_race creates and simulates a race on the basis of the inserted parameters, and returns
/// the results for post-processing.
pub fn handle_race(
    sim_pars: &SimPars,
    timestep_size: f64,
    print_debug: bool,
    tx: Option<&Sender<RaceState>>,
    realtime_factor: f64,
) -> anyhow::Result<RaceResult> {
    // create the race
    let mut race = Race::new(
        &sim_pars.race_pars,
        &sim_pars.track_pars,
        &sim_pars.driver_pars_all,
        &sim_pars.car_pars_all,
        timestep_size,
    );

    // check if sender was inserted -> in that case use real-time simulation for GUI
    let sim_realtime = tx.is_some();

    // simulate the race -> execute simulation steps until race is finished for all cars
    if !sim_realtime {
        // NORMAL SIMULATION -----------------------------------------------------------------------
        while !race.get_all_finished() {
            // simulate time step
            race.simulate_timestep();
        }
    } else {
        // REAL-TIME SIMULATION --------------------------------------------------------------------
        let mut t_race_update_print = 0.0;
        let mut t_race_update_gui = 0.0;

        while !race.get_all_finished() {
            let t_start = Instant::now();

            // simulate time step
            race.simulate_timestep();

            // print status (with a maximum of 1 Hz)
            if race.cur_racetime > t_race_update_print + 0.9999 {
                println!(
                    "INFO: Simulating... Current race time is {:.3}s, current lap is {}",
                    race.cur_racetime, race.cur_lap_leader
                );
                t_race_update_print = race.cur_racetime;
            }

            // update GUI
            if race.cur_racetime > t_race_update_gui + 1.0 / MAX_GUI_UPDATE_FREQUENCY - 0.001 {
                // create RaceState struct and set data
                let mut race_state = RaceState {
                    car_states: Vec::with_capacity(race.cars_list.len()),
                    flag_state: race.flag_state.to_owned(),
                };

                for car in race.cars_list.iter() {
                    // convert hex color to a rgb color
                    let tmp_color = car
                        .color
                        .parse::<css_color_parser::Color>()
                        .context("Could not parse hex color!")?;

                    race_state.car_states.push(CarState {
                        car_no: car.car_no,
                        driver_initials: car.driver.initials.to_owned(),
                        color: RgbColor {
                            r: tmp_color.r,
                            g: tmp_color.g,
                            b: tmp_color.b,
                        },
                        race_prog: car.sh.get_race_prog(),
                    });
                }

                // send current race state
                tx.unwrap()
                    .send(race_state)
                    .context("Failed to send race state to GUI!")?;
                t_race_update_gui = race.cur_racetime;
            }

            // sleep until time step is finished in real-time as well (calculation in ms)
            let t_sleep = (race.timestep_size * 1000.0 / realtime_factor) as i64
                - t_start.elapsed().as_millis() as i64;

            if t_sleep > 0 {
                sleep(Duration::from_millis(t_sleep as u64));
            } else {
                println!("WARNING: Could not keep up with real-time!")
            }
        }
    }

    // print debug information if indicated
    if print_debug {
        println!(
            "DEBUG: Estimated time loss for driving through the pit lane (w/o standstill): {:.2}s",
            race.track.get_pit_drive_timeloss()
        )
    }

    // return race result
    Ok(race.get_race_result())
}
