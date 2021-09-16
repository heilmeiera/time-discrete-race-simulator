use crate::core::car::{Car, CarPars};
use crate::core::driver::{Driver, DriverPars};
use crate::core::track::{Track, TrackPars};
use crate::post::race_result::{CarDriverPair, RaceResult};
use helpers::general::{argmax, argsort, SortOrder};
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;

/// * `season` - Season of the race
/// * `tot_no_laps` - Total number of laps in the race
/// * `drs_allowed_lap` - DRS activation is allowed from this lap onwards (usually second lap)
/// * `min_t_dist` - (s) Minimal temporal distance to driver in front
/// * `t_duel` - (s) Time loss applied to duelling drivers if they fight for position
/// * `t_overtake_loser` - (s) Time loss applied to the loser of an overtaking maneuver
/// * `drs_window` - (s) DRS window, usually 1.0s
/// * `use_drs` - Boolean to determine whether DRS is used in the race
/// * `participants` - List of participants (car numbers) in the current race (the respective car
/// parameters must be available)
#[derive(Debug, Deserialize, Clone)]
pub struct RacePars {
    pub season: u32,
    pub tot_no_laps: u32,
    pub drs_allowed_lap: u32,
    pub min_t_dist: f64,
    pub t_duel: f64,
    pub t_overtake_loser: f64,
    pub drs_window: f64,
    pub use_drs: bool,
    pub participants: Vec<u32>,
}

#[derive(Debug, Clone)]
pub enum FlagState {
    G,   // green
    Y,   // yellow
    Vsc, // virtual safety car
    Sc,  // safety car
    C,   // chequered
}

impl Default for FlagState {
    fn default() -> Self {
        FlagState::G
    }
}

#[derive(Debug)]
pub struct Race {
    pub timestep_size: f64,
    pub cur_racetime: f64,
    season: u32,
    pub tot_no_laps: u32,
    drs_allowed_lap: u32,
    pub cur_lap_leader: u32,
    min_t_dist: f64,
    t_duel: f64,
    t_overtake_loser: f64,
    drs_window: f64,
    use_drs: bool,
    pub flag_state: FlagState,
    pub track: Track,
    race_finished: Vec<bool>,
    pub laptimes: Vec<Vec<f64>>,
    pub racetimes: Vec<Vec<f64>>,
    cur_laptimes: Vec<f64>,
    cur_th_laptimes: Vec<f64>,
    pub cars_list: Vec<Car>,
    drivers_list: HashMap<String, Rc<Driver>>,
}

impl Race {
    pub fn new(
        race_pars: &RacePars,
        track_pars: &TrackPars,
        driver_pars_all: &HashMap<String, DriverPars>,
        car_pars_all: &HashMap<u32, CarPars>,
        timestep_size: f64,
    ) -> Race {
        // create drivers
        let mut drivers_list = HashMap::with_capacity(driver_pars_all.len());

        for (initials, driver_pars) in driver_pars_all.iter() {
            drivers_list.insert(initials.to_owned(), Rc::new(Driver::new(driver_pars)));
        }

        // create cars
        let no_cars = race_pars.participants.len();
        let mut cars_list: Vec<Car> = Vec::with_capacity(no_cars);

        for car_no in race_pars.participants.iter() {
            let car_pars_tmp = car_pars_all
                .get(car_no)
                .expect("Missing car number in car parameters!");

            cars_list.push(Car::new(
                car_pars_tmp,
                Rc::clone(
                    drivers_list
                        .get(&car_pars_tmp.strategy[0].driver_initials)
                        .expect("Could not find start driver initials in drivers list!"),
                ),
            ));
        }

        // sort cars list by car number
        cars_list.sort_unstable_by(|a, b| a.car_no.partial_cmp(&b.car_no).unwrap());

        // create race
        let mut race = Race {
            timestep_size,
            cur_racetime: 0.0,
            season: race_pars.season,
            tot_no_laps: race_pars.tot_no_laps,
            drs_allowed_lap: race_pars.drs_allowed_lap,
            cur_lap_leader: 1,
            min_t_dist: race_pars.min_t_dist,
            t_duel: race_pars.t_duel,
            t_overtake_loser: race_pars.t_overtake_loser,
            drs_window: race_pars.drs_window,
            use_drs: race_pars.use_drs,
            flag_state: FlagState::G,
            track: Track::new(track_pars),
            race_finished: vec![false; no_cars],
            laptimes: vec![vec![0.0; race_pars.tot_no_laps as usize + 1]; no_cars],
            racetimes: vec![vec![0.0; race_pars.tot_no_laps as usize + 1]; no_cars],
            cur_laptimes: vec![0.0; no_cars],
            cur_th_laptimes: vec![0.0; no_cars],
            cars_list,
            drivers_list,
        };

        // initialize race for each car
        for idx in 0..race.cars_list.len() {
            // calculate theoretical lap time for first lap
            race.calc_th_laptime(idx);

            // initialize state handler of the car
            let car = &mut race.cars_list[idx];

            let s_track_start =
                race.track.d_first_gridpos + (car.p_grid - 1) as f64 * race.track.d_per_gridpos;

            car.sh.initialize_state_handler(
                race.use_drs,
                race.track.turn_1,
                race.drs_window,
                s_track_start,
                race.track.length,
                race.track.drs_measurement_points.to_owned(),
                race.track.pit_zone,
                race.track.overtaking_zones.to_owned(),
            )
        }

        race
    }

    // ---------------------------------------------------------------------------------------------
    // MAIN METHOD ---------------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------

    /// The method simulates one time step. Execution order:
    /// 1. Increment the discretization variable (cur_racetime).
    /// 2. Calculate the current lap time for each car based on the state after the previous time
    /// step. The current lap time depends, for example, on the fuel mass, the age of the tires, the
    /// interactions between the drivers, and random influences. If a car is in standstill state
    /// during a pit stop, its lap time is infinite.
    /// 3. Update the race progress of each car for the given time step based on its current lap
    /// time.
    /// 4. Handle the situation if any car enters or leaves the pit standstill state in the current
    /// time step (if pits are located before the finish line).
    /// 5. Handle lap transitions for those cars that reached a new lap in the current time step.
    /// 6. Handle the situation if any car enters or leaves the pit standstill state in the current
    /// time step (if pits are located after the finish line).
    /// 7. Check if any car switches to a new state for the next time step.
    pub fn simulate_timestep(&mut self) {
        // increment discretization variable
        self.cur_racetime += self.timestep_size;

        // adjust current lap times such that flags, DRS etc. are considered and minimum distances
        // are kept
        self.calc_cur_laptimes();

        // update race progress
        for (i, car) in self.cars_list.iter_mut().enumerate() {
            car.sh
                .update_race_prog(self.cur_laptimes[i], self.timestep_size)
        }

        // handle pit stop standstill part (if pits are located in front of the finish line -
        // uncommon case)
        if !self.track.pits_aft_finishline {
            self.handle_pit_standstill()
        }

        // handle lap transitions
        self.handle_lap_transitions();

        // handle pit stop standstill part (if pits are located behind the finish line - common
        // case)
        if self.track.pits_aft_finishline {
            self.handle_pit_standstill()
        }

        // handle state transitions
        self.handle_state_transitions();
    }

    // ---------------------------------------------------------------------------------------------
    // RACE SIMULATOR PARTS ------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------

    /// The method calculates the laptime a driver-car combo can theoretically drive in the current
    /// lap on a free race track, i.e. if the combo is not blocked by another car, safety car etc.
    fn calc_th_laptime(&mut self, idx: usize) {
        // consider base lap time as well as driver-specific time losses and gains based on tire
        // degradation and fuel mass loss
        self.cur_th_laptimes[idx] = self.track.t_q
            + self.track.t_gap_racepace
            + self.cars_list[idx].calc_basic_timeloss(self.track.s_mass)
    }

    /// The method adjusts the theoretical lap times such that environmental effects are considered.
    /// This includes race start, flag state, duelling between two drivers, DRS, and pit time
    /// losses. Furthermore, the velocity is decreased (lap time is increased) if a car is too
    /// close to a car in front (if it does not currently overtake it). This is actually the
    /// overtaking implementation: either a car must keep the minimum distance to the car in front
    /// or it is allowed to try to overtake.
    fn calc_cur_laptimes(&mut self) {
        for (i, car) in self.cars_list.iter().enumerate() {
            // reset lap time
            self.cur_laptimes[i] = self.cur_th_laptimes[i];

            // consider race start from a standstill (time loss due to grid position is already
            // included by a negative value of the s coordinate at the race start)
            if car.sh.start_act {
                self.cur_laptimes[i] += self.track.t_loss_firstlap / self.track.turn_1_lap_frac
            }

            // consider lap time loss caused by duelling (fully applied in overtaking zones)
            if car.sh.duel_act {
                self.cur_laptimes[i] += self.t_duel / self.track.overtaking_zones_lap_frac;
            }

            // consider lap time gain due to DRS (fully applied in overtaking zones, t_drs_effect is
            // negative)
            if car.sh.drs_act {
                self.cur_laptimes[i] +=
                    self.track.t_drseffect / self.track.overtaking_zones_lap_frac;
            }

            // consider time loss due to a pit stop
            if car.sh.pit_act {
                if !car.sh.pit_standstill_act {
                    // case 1: driving through the pit lane (not in standstill, state Pitlane), lap
                    // time is virtually increased to consider that real pit lane length can be
                    // greater than the projection on the track's s coordinate
                    self.cur_laptimes[i] = self.track.length / self.track.pit_speedlimit
                        * self.track.real_length_pit_zone
                        / self.track.track_length_pit_zone;
                } else {
                    // case 2: car is in standstill (state PitStandstill) at the beginning of the
                    // current time step
                    if let Some(t_driving) = car.sh.check_leaves_standstill(self.timestep_size) {
                        // case 2a: car returns from standstill to driving within this time step
                        // (state PitStandstill is deactivated later in the handle_pit_standstill
                        // method), lap time is virtually increased to consider that real pit lane
                        // length can be greater than the projection on the track's s coordinate
                        self.cur_laptimes[i] = self.track.length / self.track.pit_speedlimit
                            * self.track.real_length_pit_zone
                            / self.track.track_length_pit_zone
                            * self.timestep_size
                            / t_driving;
                    } else {
                        // case 2b: car stays in standstill for the entire time step
                        self.cur_laptimes[i] = f64::INFINITY;
                    }
                }
            }

            // consider current flag state (minimum lap time) if car is not in the pit lane
            if !car.sh.pit_act && self.cur_laptimes[i] < self.get_min_laptime_flag_state() {
                self.cur_laptimes[i] = self.get_min_laptime_flag_state()
            }
        }

        // ADJUST LAP TIME IF TOO CLOSE TO CAR IN FRONT AND NOT IN OVERTAKING STATE ----------------
        // Using the car with the biggest gap in front as a starting point has the advantage that
        // its velocity/lap time does not need to be adjusted (at least that is assumed) and
        // therefore no extra loop is required.

        // iterate through the cars pair-wise and check their temporal distance
        let idxs_sorted = self.get_idx_list_sorted_by_biggest_gap();
        let car_pair_idxs_list = self.get_car_pair_idxs_list(&idxs_sorted, true);

        for pair_idxs in car_pair_idxs_list.iter() {
            // calculate temporal distance as it is expected to be at the end of the current time
            // step
            let delta_t_proj =
                self.calc_projected_delta_t(pair_idxs[0], pair_idxs[1], self.timestep_size);

            // if temporal distance would be too small at the end of the time step increase lap time
            // of the rear car (at least to the lap time of the car in front if required minimum
            // distance is currently kept) -> this also catches the case that the rear car overtakes
            // the car in front by accident due to a suddenly reduced lap time
            if !self.cars_list[pair_idxs[1]].sh.overtaking_act
                && !self.cars_list[pair_idxs[0]].sh.pit_act
                && delta_t_proj < self.min_t_dist
            {
                // calculate current temporal distance to determine new lap time
                let delta_t_cur = self.calc_projected_delta_t(pair_idxs[0], pair_idxs[1], 0.0);

                // calculate time that must be added to increase temporal distance to car in front
                // to the desired value min_t_dist within 3 seconds
                let t_gap_add =
                    (self.min_t_dist - delta_t_cur) / 3.0 * self.cur_laptimes[pair_idxs[1]];

                // apply lap time (if it is not already slow enough)
                if self.cur_laptimes[pair_idxs[1]] < self.cur_laptimes[pair_idxs[0]] + t_gap_add {
                    self.cur_laptimes[pair_idxs[1]] = self.cur_laptimes[pair_idxs[0]] + t_gap_add
                }
            }
        }
    }

    /// The method returns a minimum lap time that must be kept in dependence of the current flag
    /// state.
    fn get_min_laptime_flag_state(&self) -> f64 {
        match self.flag_state {
            FlagState::Y => (self.track.t_q + self.track.t_gap_racepace) * 1.1,
            FlagState::Vsc => (self.track.t_q + self.track.t_gap_racepace) * 1.4,
            FlagState::Sc => (self.track.t_q + self.track.t_gap_racepace) * 1.4,
            _ => 0.0,
        }
    }

    /// The method returns an index list that is sorted in a way such that the car with the biggest
    /// spatial gap in front of it is located at the beginning of the list. The remaining cars
    /// follow in the order in which they are driving on the track.
    fn get_idx_list_sorted_by_biggest_gap(&self) -> Vec<usize> {
        // calculate gaps between the car pairs
        let mut idx_list_sorted = self.get_car_order_on_track();
        let car_pair_idxs_list = self.get_car_pair_idxs_list(&idx_list_sorted, false);

        let delta_lap_fracs: Vec<f64> = car_pair_idxs_list
            .iter()
            .map(|x| self.calc_projected_delta_lap_frac(x[0], x[1], 0.0))
            .collect();

        // find biggest gap between two cars
        let pair_idx_biggest_gap = argmax(&delta_lap_fracs);

        // rotate idx_list_sorted to start with the index of the car with the biggest gap in front
        // of it
        let start_idx = (pair_idx_biggest_gap + 1) % self.cars_list.len();
        idx_list_sorted.rotate_left(start_idx);

        idx_list_sorted
    }

    /// The method checks if any car reaches the pit location within the current time step and
    /// activates the pit standstill state in that case. If a car is already in standstill state,
    /// the method assures that the standstill time is increased and that it leaves the state as
    /// soon as it exceedes the target time. The pit stop itself (i.e. refueling and tire change) is
    /// nevertheless executed in the handle_lap_transition method to avoid issues due to wrong tire
    /// age etc.
    fn handle_pit_standstill(&mut self) {
        for (i, car) in self.cars_list.iter_mut().enumerate() {
            // check for possible activation of standstill state if car is within the pit and not
            // already in standstill state
            if car.sh.pit_act && !car.sh.pit_standstill_act {
                // calculate time part that was driven before crossing the pit location if the car
                // crossed the pit location within the current time step, else continue
                let t_part_drive: f64;

                if car.sh.get_s_track_passed_this_step(car.pit_location) {
                    let (s_track_prev, s_track_cur) = car.sh.get_s_tracks();

                    if !self.track.pits_aft_finishline {
                        // drive time part is known without issues caused by a possible lap
                        // transition
                        t_part_drive = (car.pit_location - s_track_prev) / self.track.length
                            * self.cur_laptimes[i];
                    } else {
                        // standstill time part is known without issues caused by a possible lap
                        // transition -> subtract it from the time step size
                        t_part_drive = self.timestep_size
                            - (s_track_cur - car.pit_location) / self.track.length
                                * self.cur_laptimes[i];
                    }
                } else {
                    continue;
                }

                // below this line we handle the case that the car enters the standstill state
                // within the current step ---------------------------------------------------------

                // determine standstill target time for current pit stop
                let compl_lap_cur = car.sh.get_compl_lap();
                let t_standstill_target = if self.track.pits_aft_finishline {
                    car.t_add_pit_standstill(compl_lap_cur)
                } else {
                    car.t_add_pit_standstill(compl_lap_cur + 1)
                };

                // set car state to pit standstill and set standstill time that was already achieved
                car.sh
                    .act_pit_standstill(self.timestep_size - t_part_drive, t_standstill_target);

                // update race progress of the car such that it is placed exactly at the pit
                // location
                car.sh.set_s_track(car.pit_location);
            } else if car.sh.pit_standstill_act {
                // if standstill is active currently, it must be checked if the car stays or leaves
                // it within the current time step
                let leaves_standstill =
                    car.sh.check_leaves_standstill(self.timestep_size).is_some();

                if !leaves_standstill {
                    // car remains in standstill, therefore increment standstill time
                    car.sh.increment_t_standstill(self.timestep_size)
                } else {
                    // car leaves standstill state within current time step
                    car.sh.deact_pit_standstill()
                }
            }
        }
    }

    /// The method is responsible to handle the transition between two laps correctly. The
    /// transition includes a jump in the lap time, for example.
    fn handle_lap_transitions(&mut self) {
        // check at first if race was finished by any car such that checkered flag can be considered
        // in the loop afterward (required since cars cannot complete all laps if they were lapped,
        // for example)
        for car in self.cars_list.iter() {
            let compl_lap_cur = car.sh.get_compl_lap();

            if compl_lap_cur >= self.cur_lap_leader {
                self.cur_lap_leader = compl_lap_cur + 1
            }
        }

        if self.cur_lap_leader > self.tot_no_laps && !matches!(self.flag_state, FlagState::C) {
            self.flag_state = FlagState::C
        }

        // check for all cars if they jumped into a new lap within the current time step
        for i in 0..self.cars_list.len() {
            let car = &mut self.cars_list[i];

            if car.sh.get_new_lap() {
                // calculate the part of the current time step that was driven before crossing the
                // finish line
                let lap_frac_prev = car.sh.get_lap_fracs().0;
                let t_part_old = (1.0 - lap_frac_prev) * self.cur_laptimes[i];

                // update lap time and race time arrays (if laps are part of the race)
                let compl_lap_cur = car.sh.get_compl_lap();

                if compl_lap_cur <= self.tot_no_laps {
                    self.laptimes[i][compl_lap_cur as usize] =
                        self.cur_racetime - self.timestep_size + t_part_old
                            - self.racetimes[i][compl_lap_cur as usize - 1];
                    self.racetimes[i][compl_lap_cur as usize] = self.racetimes[i]
                        [compl_lap_cur as usize - 1]
                        + self.laptimes[i][compl_lap_cur as usize];
                }

                // set race finished for current car if it crosses the line after the chequered flag
                // got active
                if matches!(self.flag_state, FlagState::C) {
                    self.race_finished[i] = true
                }

                // increase car age by a lap
                car.drive_lap();

                // perform pit stop (if pit_act is true) when crossing the finish line (this is done
                // here to avoid wrong tire ages even though the standstill is performed either
                // before or after the finish line)
                if car.sh.pit_act {
                    car.perform_pitstop(compl_lap_cur, &self.drivers_list)
                }

                // update theoretical lap time
                self.calc_th_laptime(i);
            }
        }
    }

    /// The method prepares the required data for the car statemachine state-transition check, and
    /// calls it.
    fn handle_state_transitions(&mut self) {
        // calculate gaps between the car pairs and check if rear car laps the car in front
        let idxs_sorted = self.get_car_order_on_track();
        let car_pair_idxs_list = self.get_car_pair_idxs_list(&idxs_sorted, false);

        let mut delta_ts = vec![0.0; self.cars_list.len()];
        let mut lapping = vec![false; self.cars_list.len()];

        for (i, pair_idxs) in car_pair_idxs_list.iter().enumerate() {
            delta_ts[i] = self.calc_projected_delta_t(pair_idxs[0], pair_idxs[1], 0.0);

            // race start is handled correctly since get_race_prog can be negative
            if self.cars_list[pair_idxs[0]].sh.get_race_prog()
                < self.cars_list[pair_idxs[1]].sh.get_race_prog()
            {
                lapping[i] = true;
            }
        }

        // check for state transitions (always for the rear car)
        for (i, pair_idxs) in car_pair_idxs_list.iter().enumerate() {
            // get lap fraction of the car
            let compl_lap_cur = self.cars_list[pair_idxs[1]].sh.get_compl_lap();

            // set index j such that it points to the temporal distance behind the current rear
            // car
            let j = (i + 1) % car_pair_idxs_list.len();

            // check for state transition of the car
            let pit_this_lap = self.cars_list[pair_idxs[1]].pit_this_lap(compl_lap_cur + 1);

            self.cars_list[pair_idxs[1]].sh.check_state_transition(
                delta_ts[i],
                delta_ts[j],
                pit_this_lap,
                lapping[i],
                lapping[j],
                &self.flag_state,
                self.cur_lap_leader,
                self.drs_allowed_lap,
            )
        }
    }

    // ---------------------------------------------------------------------------------------------
    // METHODS (HELPERS) ---------------------------------------------------------------------------
    // ---------------------------------------------------------------------------------------------

    /// get_car_order_on_track returns the indices of the cars on the track in the correct order
    /// (sorted by descending s coordinate).
    fn get_car_order_on_track(&self) -> Vec<usize> {
        // get s coordinates
        let s_tracks_cur: Vec<f64> = self
            .cars_list
            .iter()
            .map(|car| car.sh.get_s_tracks().1)
            .collect();

        // get indices that sort the vector in a descending order
        argsort(&s_tracks_cur, SortOrder::Descending)
    }

    /// calc_projected_delta_t calculates the temporal distance between two cars, i.e. the time it
    /// takes for the rear car to pass the point on the track at which the front car currently is.
    /// The velocity/lap time used for the duration calculation is based on the rear car. If
    /// timestep_size is greater than 0.0, both cars are projected for that time step into the
    /// future before calculating their temporal distance.
    pub fn calc_projected_delta_t(
        &self,
        idx_front: usize,
        idx_rear: usize,
        timestep_size: f64,
    ) -> f64 {
        let delta_lap_frac = self.calc_projected_delta_lap_frac(idx_front, idx_rear, timestep_size);
        delta_lap_frac * self.cur_laptimes[idx_rear]
    }

    /// calc_projected_delta_lap_frac calculates the spatial distance between two cars considering
    /// lap fraction, no complete laps. If timestep_size is greater than 0.0, both cars are
    /// projected for that time step into the future before calculating their spatial distance
    /// (based on their current lap times).
    fn calc_projected_delta_lap_frac(
        &self,
        idx_front: usize,
        idx_rear: usize,
        timestep_size: f64,
    ) -> f64 {
        // get lap fractions
        let mut lap_frac_cur_front = self.cars_list[idx_front].sh.get_lap_fracs().1;
        let mut lap_frac_cur_rear = self.cars_list[idx_rear].sh.get_lap_fracs().1;

        // virtually increase race progress
        lap_frac_cur_front += timestep_size / self.cur_laptimes[idx_front];
        lap_frac_cur_rear += timestep_size / self.cur_laptimes[idx_rear];

        if lap_frac_cur_front >= 1.0 {
            lap_frac_cur_front -= 1.0
        }

        if lap_frac_cur_rear >= 1.0 {
            lap_frac_cur_rear -= 1.0
        }

        // calculate spatial distance between the two cars
        if lap_frac_cur_front >= lap_frac_cur_rear {
            lap_frac_cur_front - lap_frac_cur_rear
        } else {
            lap_frac_cur_front + 1.0 - lap_frac_cur_rear
        }
    }

    /// get_car_pair_idxs_list creates a list of car index pairs (idx front, idx rear) that can be
    /// iterated.
    fn get_car_pair_idxs_list(&self, idxs: &[usize], del_last_pair: bool) -> Vec<[usize; 2]> {
        let mut car_pair_idxs_list = vec![[0; 2]; idxs.len()];

        for i in 0..idxs.len() {
            car_pair_idxs_list[i][0] = idxs[i];
            car_pair_idxs_list[i][1] = idxs[(i + 1) % idxs.len()];
        }

        // delete last pair if indicated
        if del_last_pair {
            car_pair_idxs_list.remove(car_pair_idxs_list.len() - 1);
        }

        car_pair_idxs_list
    }

    /// get_all_finished checks if all race participants have finished the race.
    pub fn get_all_finished(&self) -> bool {
        self.race_finished.iter().all(|&x| x)
    }

    /// get_race_result returns a race result struct of the race.
    pub fn get_race_result(&self) -> RaceResult {
        RaceResult {
            tot_no_laps: self.tot_no_laps,
            car_driver_pairs: self
                .cars_list
                .iter()
                .map(|car| CarDriverPair {
                    car_no: car.car_no,
                    driver_initials: car.driver.initials.to_owned(),
                })
                .collect(),
            laptimes: self.laptimes.to_owned(),
            racetimes: self.racetimes.to_owned(),
        }
    }
}
