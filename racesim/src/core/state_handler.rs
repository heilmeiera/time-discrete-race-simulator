use crate::core::race::FlagState;

#[derive(Debug)]
pub enum State {
    Racestart,
    NormalZone,
    OvertakingZone,
    Pitlane,
    PitStandstill,
}

/// The StateHandler contains a statemachine to track if a car is allowed to overtake, use DRS, is
/// in pit, etc. Furthermore, it keeps track of the car's current race progress.
///
/// Possible statemachine states:
/// * `Racestart` -> active only once at the beginning of the race
/// * `NormalZone` -> car is between two overtaking zones
/// * `OvertakingZone` -> car is in an overtaking zone
/// * `Pitlane` -> driving through the pit lane
/// * `PitStandstill` -> reachable from state Pitlane by external method call, returning to state
/// `Pitlane` by external method call)
///
/// `Racestart` is a special state as it cannot be reached again once it was left. It allows
/// overtaking until turn 1, then switches to the correct subsequent state (`NormalZone` or
/// `OvertakingZone`). The minimum distances must not be kept while race start is active.
#[derive(Debug)]
pub struct StateHandler {
    // parameters
    use_drs: bool,
    turn1: f64,
    drs_measurement_points: Vec<f64>,
    drs_window: f64,
    overtaking_zones: Vec<[f64; 2]>, // [start, end] for each overtaking zone
    pit_zone: [f64; 2],              // [start, end]
    track_length: f64,
    // variables related to both, statemachine and race progress handling
    s_track_prev: f64,
    s_track_cur: f64,
    // variables related to the statemachine
    act_zone_idx: usize, // index of overtaking zone that is active next (state NormalZone) or currently (state overtaking)
    first_zone_info: [usize; 2], // two indices indicating which overtaking zone and which side (start or end) of the zone comes first after the finish line
    state: State,
    t_standstill: f64, // used to handle the standstill correctly during a pit stop
    t_standstill_target: f64, // used to handle the standstill correctly during a pit stop
    in_drs_window: bool,
    pub start_act: bool,
    pub drs_act: bool,
    pub overtaking_act: bool,
    pub pit_act: bool,
    pub pit_standstill_act: bool,
    pub duel_act: bool,
    // variables related to the race progress handling
    compl_lap_prev: u32,
    compl_lap_cur: u32,
}

impl StateHandler {
    #[allow(clippy::too_many_arguments)]
    pub fn initialize_state_handler(
        &mut self,
        use_drs: bool,
        turn1: f64,
        drs_window: f64,
        s_track_start: f64,
        track_length: f64,
        drs_measurement_points: Vec<f64>,
        pit_zone: [f64; 2],
        overtaking_zones: Vec<[f64; 2]>,
    ) {
        // initialize parameters
        self.use_drs = use_drs;
        self.turn1 = turn1;
        self.drs_measurement_points = drs_measurement_points;
        self.drs_window = drs_window;
        self.overtaking_zones = overtaking_zones;
        self.pit_zone = pit_zone;
        self.track_length = track_length;

        // initialize s coordinate variables (can be negative at the race start)
        self.s_track_prev = s_track_start;
        self.s_track_cur = s_track_start;

        // determine first zone boundary (i: zone index, j: side, i.e. start or end) after the
        // finish line
        let mut s_min = f64::INFINITY;

        for (i, zone) in self.overtaking_zones.iter().enumerate() {
            for (j, &s_tmp) in zone.iter().enumerate() {
                if s_tmp < s_min {
                    s_min = s_tmp;
                    self.first_zone_info[0] = i;
                    self.first_zone_info[1] = j;
                }
            }
        }
    }

    pub fn get_s_track_passed_this_step(&self, s_track: f64) -> bool {
        // determine if car crossed the finish line within the current time step
        let new_lap = self.get_new_lap();

        // check if car passed the inserted s coordinate within this time step
        if (self.s_track_prev < s_track || new_lap) && s_track <= self.s_track_cur
            || self.s_track_prev < s_track && (s_track <= self.s_track_cur || new_lap)
        {
            return true;
        }
        false
    }

    /// check_state_transition checks if the car jumps from one state into another. A car-driver
    /// combo is in a duel for the position if it is within an overtaking zone and if delta_t to
    /// the front or rear car is within the DRS window (not during lapping). The method returns the
    /// lap fraction that is driven under the new state such that it can be considered in the next
    /// time step.
    #[allow(clippy::too_many_arguments)]
    pub fn check_state_transition(
        &mut self,
        delta_t_front: f64,
        delta_t_rear: f64,
        pit_this_lap: bool,
        lapping_front: bool,
        lapping_rear: bool,
        flag_state: &FlagState,
        cur_lap_leader: u32,
        drs_allowed_lap: u32,
    ) {
        match self.state {
            // START -------------------------------------------------------------------------------
            State::Racestart => {
                if self.get_s_track_passed_this_step(self.turn1) {
                    // subsequent state and zone are not defined and must therefore be determined
                    // based on current position
                    let (state, act_zone_idx) = self.get_act_state_and_zone();
                    self.state = state;
                    self.act_zone_idx = act_zone_idx;
                    self.start_act = false;
                    self.overtaking_act = false;
                }
            }

            // NORMALZONE --------------------------------------------------------------------------
            State::NormalZone => {
                if self.get_s_track_passed_this_step(self.drs_measurement_points[self.act_zone_idx])
                {
                    // check if DRS is enabled for the next overtaking zone
                    if delta_t_front <= self.drs_window {
                        self.in_drs_window = true
                    }
                }
                if pit_this_lap && self.get_s_track_passed_this_step(self.pit_zone[0]) {
                    self.state = State::Pitlane;
                    self.pit_act = true;
                    self.in_drs_window = false;
                } else if self
                    .get_s_track_passed_this_step(self.overtaking_zones[self.act_zone_idx][0])
                {
                    self.state = State::OvertakingZone;
                    if !(matches!(flag_state, FlagState::Vsc | FlagState::Sc)) {
                        self.overtaking_act = true;

                        // check if DRS gets activated
                        if self.use_drs && cur_lap_leader >= drs_allowed_lap && self.in_drs_window {
                            self.drs_act = true;
                        }

                        // check duelling (not applied in case of lapping)
                        if delta_t_front <= self.drs_window && !lapping_front
                            || delta_t_rear <= self.drs_window && !lapping_rear
                        {
                            self.duel_act = true
                        }
                    }
                    self.in_drs_window = false;
                }
            }

            // OVERTAKINGZONE ----------------------------------------------------------------------
            State::OvertakingZone => {
                if pit_this_lap && self.get_s_track_passed_this_step(self.pit_zone[0]) {
                    self.state = State::Pitlane;
                    self.pit_act = true;
                    self.drs_act = false;
                    self.overtaking_act = false;
                    self.duel_act = false;
                } else if self
                    .get_s_track_passed_this_step(self.overtaking_zones[self.act_zone_idx][1])
                    || matches!(flag_state, FlagState::Vsc | FlagState::Sc)
                {
                    self.state = State::NormalZone;
                    self.act_zone_idx = (self.act_zone_idx + 1) % self.overtaking_zones.len(); // set next zone
                    self.drs_act = false;
                    self.overtaking_act = false;
                    self.duel_act = false;
                }
            }

            // PIT ---------------------------------------------------------------------------------
            State::Pitlane => {
                if self.get_s_track_passed_this_step(self.pit_zone[1]) {
                    // subsequent state and zone are not defined and must therefore be determined
                    // based on current position
                    let (state, act_zone_idx) = self.get_act_state_and_zone();
                    self.state = state;
                    self.act_zone_idx = act_zone_idx;
                    self.pit_act = false;
                }
            }

            // PIT STANDSTILL ----------------------------------------------------------------------
            // activation and deactivation happens by external method calls since these cases must
            // be handled within the race class itself
            State::PitStandstill => {}
        }
    }

    /// act_pit_standstill is used to activate the pit standstill state from within the race class
    /// during a pit stop.
    pub fn act_pit_standstill(&mut self, t_standstill: f64, t_standstill_target: f64) {
        if !matches!(self.state, State::Pitlane) {
            panic!("Tried to enter pit standstill state without being in pit state!")
        }

        self.state = State::PitStandstill;
        self.pit_standstill_act = true;
        self.t_standstill = t_standstill;
        self.t_standstill_target = t_standstill_target;
    }

    /// deact_pit_standstill is used to deactivate the pit standstill state from within the race
    /// class during a pit stop.
    pub fn deact_pit_standstill(&mut self) {
        if !matches!(self.state, State::PitStandstill) {
            panic!("Tried to revert to pit state without being in pit standstill state!")
        }

        self.state = State::Pitlane;
        self.pit_standstill_act = false;
        self.t_standstill = 0.0;
        self.t_standstill_target = 0.0;
    }

    /// increment_t_standstill is used to increment the standstill time from within the race class.
    pub fn increment_t_standstill(&mut self, timestep_size: f64) {
        if !matches!(self.state, State::PitStandstill) {
            panic!("Tried to increment standstill time without being in standstill state!")
        }

        self.t_standstill += timestep_size;
    }

    /// check_leaves_standstill is used to check if the car leaves the standstill within this time
    /// step, and if he does, it also returns the time he already drives within the current time
    /// step.
    pub fn check_leaves_standstill(&self, timestep_size: f64) -> Option<f64> {
        if !matches!(self.state, State::PitStandstill) {
            panic!("Tried to check if car leaves standstill without being in standstill state!")
        }

        if self.t_standstill + timestep_size <= self.t_standstill_target {
            None
        } else {
            Some(self.t_standstill + timestep_size - self.t_standstill_target)
        }
    }

    /// get_act_state_and_zone returns the correct car state for the current position, when the
    /// state is unclear, e.g. after a pit stop.
    pub fn get_act_state_and_zone(&self) -> (State, usize) {
        // loop through the zone boundaries in the order as they appear after the finish line and
        // check if s_track_cur is in front of the corresponding s values
        let mut tmp_zone_idx = self.first_zone_info[0];
        let mut tmp_side_idx = self.first_zone_info[1];

        loop {
            // if s_track_cur is in front of current zone boundary break loop
            if self.s_track_cur < self.overtaking_zones[tmp_zone_idx][tmp_side_idx] {
                break;
            }

            // increment tmp_side_idx (can only be 0 or 1) and tmp_zone_idx (value range depending
            // on number of overtaking zones) if applicable
            tmp_side_idx = (tmp_side_idx + 1) % 2;
            if tmp_side_idx == 0 {
                tmp_zone_idx = (tmp_zone_idx + 1) % self.overtaking_zones.len()
            }

            // if the temporary indices reach their start values again, the loop must be escaped
            // (the car is then located behind the last zone boundary and in front of the finish
            // line, which is why he is not recognized as actually being in front of the first zone
            // boundary in the according check) -> tmpZoneIdx and tmpSideIdx are correct after
            // escaping the loop
            if tmp_zone_idx == self.first_zone_info[0] && tmp_side_idx == self.first_zone_info[1] {
                break;
            }
        }

        // if car is in front of next zone (tmp_side_idx 0), he should be in state NormalZone,
        // otherwise in state OvertakingZone
        let act_state = if tmp_side_idx == 0 {
            State::NormalZone
        } else {
            State::OvertakingZone
        };
        let act_zone_idx = tmp_zone_idx;

        (act_state, act_zone_idx)
    }

    /// get_lap_fracs returns the lap fractions in the previous and the current time step (always
    /// positive, also at the race start).
    pub fn get_lap_fracs(&self) -> (f64, f64) {
        let lap_frac_prev = if self.s_track_prev < 0.0 {
            (self.s_track_prev + self.track_length) / self.track_length
        } else {
            self.s_track_prev / self.track_length
        };

        let lap_frac_cur = if self.s_track_cur < 0.0 {
            (self.s_track_cur + self.s_track_cur) / self.track_length
        } else {
            self.s_track_cur / self.track_length
        };

        (lap_frac_prev, lap_frac_cur)
    }

    /// get_s_tracks returns the s coordinates in the previous and the current time step (always
    /// positive, also at the race start).
    pub fn get_s_tracks(&self) -> (f64, f64) {
        let s_track_prev = if self.s_track_prev < 0.0 {
            self.s_track_prev + self.track_length
        } else {
            self.s_track_prev
        };

        let s_track_cur = if self.s_track_cur < 0.0 {
            self.s_track_cur + self.s_track_cur
        } else {
            self.s_track_cur
        };

        (s_track_prev, s_track_cur)
    }

    /// get_compl_lap returns the number of completed race laps in the current time step (zero at
    /// the race start).
    pub fn get_compl_lap(&self) -> u32 {
        self.compl_lap_cur
    }

    /// get_race_prog returns the current race progress (can be negative at the race start).
    pub fn get_race_prog(&self) -> f64 {
        self.compl_lap_cur as f64 + self.s_track_cur / self.track_length
    }

    /// get_new_lap returns a bool indicating if the car jumped into a new lap from previous to
    /// current time step (is not true when switching from negative to positive side during the race
    /// start).
    pub fn get_new_lap(&self) -> bool {
        self.compl_lap_cur > self.compl_lap_prev
    }

    /// set_s_track is used to set a specific s coordinate, e.g. after transitioning into a new lap
    /// with a new lap time. This method must be used with care since it directly affects the s
    /// coordinate.
    pub fn set_s_track(&mut self, s_track_cur: f64) {
        // setting a negative s coordinate is only required for the race start (which is handled
        // within the init method) and therefore not allowed here
        if !(0.0 <= s_track_cur && s_track_cur < self.track_length) {
            panic!(
                "Distance s_track_cur must be in [0.0, track_length[, but is {:.3}m!",
                s_track_cur
            )
        }
        self.s_track_cur = s_track_cur;
    }

    /// update_race_prog increments the race progress according to the current lap time.
    pub fn update_race_prog(&mut self, cur_laptime: f64, timestep_size: f64) {
        // update previous state
        self.compl_lap_prev = self.compl_lap_cur;
        self.s_track_prev = self.s_track_cur;

        // update current state
        self.s_track_cur += timestep_size / cur_laptime * self.track_length;

        // check if car jumped into new lap within current time step (negative s_track at the race
        // start will not lead to a new lap)
        if self.s_track_cur >= self.track_length {
            self.compl_lap_cur += 1;
            self.s_track_cur -= self.track_length;
        }
    }
}

impl Default for StateHandler {
    fn default() -> Self {
        StateHandler {
            use_drs: false,
            turn1: 0.0,
            drs_measurement_points: vec![],
            drs_window: 0.0,
            overtaking_zones: vec![],
            pit_zone: [0.0, 0.0],
            track_length: 0.0,
            s_track_prev: 0.0,
            s_track_cur: 0.0,
            act_zone_idx: 0,
            first_zone_info: [0, 0],
            state: State::Racestart,
            t_standstill: 0.0,
            t_standstill_target: 0.0,
            in_drs_window: false,
            start_act: true,
            drs_act: false,
            overtaking_act: true, // overtaking is allowed at the race start
            pit_act: false,
            pit_standstill_act: false,
            duel_act: false,
            compl_lap_prev: 0,
            compl_lap_cur: 0,
        }
    }
}
