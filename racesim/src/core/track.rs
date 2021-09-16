use serde::Deserialize;

/// * `name` - Track name
/// * `t_q` - (s) Best qualifying lap time
/// * `t_gap_racepace` - (s) Estimated gap between t_q and best race lap time (due to engine mode
/// etc.)
/// * `s_mass` - (s/kg) Lap time mass sensitivity
/// * `t_drseffect` - (s) Lap time reduction when using DRS in all available DRS zones (negative)
/// * `pit_speedlimit` - (m/s) Speed limit when driving through the pit lane
/// * `t_loss_firstlap` - (s) Lap time loss due to the start from standstill
/// * `d_per_gridpos` - (m) Distance between two grid positions (negative)
/// * `d_first_gridpos` - (m) Distance between the first grid position and the finish line (can be
/// negative or positive)
/// * `length` - (m) Length of the track
/// * `real_length_pit_zone`- (m) Real length of pit zone (required to virtually adjust pit lane
/// speed such that a shorter or longer pit lane can be considered)
/// * `s12` - (m) Boundary between sectors 1 and 2
/// * `s23` - (m) Boundary between sectors 2 and 3
/// * `drs_measurement_points` - (m) DRS measurement points
/// * `turn_1` - (m) Distance between finish line and the first corner of the track
/// * `pit_zone` - (m) Start and end of the pit zone (in track coordinates)
/// * `pits_aft_finishline` - True if pits are located after the finish line, false if located
/// before
/// * `overtaking_zones` - (m) Start and end of the overtaking zones
#[derive(Debug, Deserialize, Clone)]
pub struct TrackPars {
    pub name: String,
    pub t_q: f64,
    pub t_gap_racepace: f64,
    pub s_mass: f64,
    pub t_drseffect: f64,
    pub pit_speedlimit: f64,
    pub t_loss_firstlap: f64,
    pub d_per_gridpos: f64,
    pub d_first_gridpos: f64,
    pub length: f64,
    pub real_length_pit_zone: f64,
    pub s12: f64,
    pub s23: f64,
    pub drs_measurement_points: Vec<f64>,
    pub turn_1: f64,
    pub pit_zone: [f64; 2],
    pub pits_aft_finishline: bool,
    pub overtaking_zones: Vec<[f64; 2]>,
}

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub t_q: f64,
    pub t_gap_racepace: f64,
    pub s_mass: f64,
    pub t_drseffect: f64,
    pub pit_speedlimit: f64,
    pub t_loss_firstlap: f64,
    pub d_per_gridpos: f64,
    pub d_first_gridpos: f64,
    pub length: f64,
    pub real_length_pit_zone: f64,
    pub track_length_pit_zone: f64,
    pub s12: f64,
    pub s23: f64,
    pub drs_measurement_points: Vec<f64>,
    pub turn_1: f64,
    pub turn_1_lap_frac: f64,
    pub pit_zone: [f64; 2],
    pub pits_aft_finishline: bool,
    pub overtaking_zones: Vec<[f64; 2]>,
    pub overtaking_zones_lap_frac: f64,
}

impl Track {
    pub fn new(track_pars: &TrackPars) -> Track {
        // determine track distance that is covered by the pit lane when driving through it
        let track_length_pit_zone = if track_pars.pit_zone[0] < track_pars.pit_zone[1] {
            track_pars.pit_zone[1] - track_pars.pit_zone[0]
        } else {
            track_pars.length - track_pars.pit_zone[0] + track_pars.pit_zone[1]
        };

        // calculate overtaking zones lap fraction
        let mut len_overtaking_zones = 0.0;

        for overtaking_zone in track_pars.overtaking_zones.iter() {
            len_overtaking_zones += if overtaking_zone[0] < overtaking_zone[1] {
                overtaking_zone[1] - overtaking_zone[0]
            } else {
                track_pars.length - overtaking_zone[0] + overtaking_zone[1]
            };
        }

        let overtaking_zones_lap_frac = len_overtaking_zones / track_pars.length;

        // calculate turn 1 lap fraction
        let turn_1_lap_frac = (track_pars.turn_1 - track_pars.d_first_gridpos) / track_pars.length;

        // create track
        Track {
            name: track_pars.name.to_owned(),
            t_q: track_pars.t_q,
            t_gap_racepace: track_pars.t_gap_racepace,
            s_mass: track_pars.s_mass,
            t_drseffect: track_pars.t_drseffect,
            pit_speedlimit: track_pars.pit_speedlimit,
            t_loss_firstlap: track_pars.t_loss_firstlap,
            d_per_gridpos: track_pars.d_per_gridpos,
            d_first_gridpos: track_pars.d_first_gridpos,
            length: track_pars.length,
            real_length_pit_zone: track_pars.real_length_pit_zone,
            track_length_pit_zone,
            s12: track_pars.s12,
            s23: track_pars.s23,
            drs_measurement_points: track_pars.drs_measurement_points.to_owned(),
            turn_1: track_pars.turn_1,
            turn_1_lap_frac,
            overtaking_zones_lap_frac,
            pits_aft_finishline: track_pars.pits_aft_finishline,
            pit_zone: track_pars.pit_zone,
            overtaking_zones: track_pars.overtaking_zones.to_owned(),
        }
    }

    /// The method returns the approximate time loss when driving through the pit lane.
    pub fn get_pit_drive_timeloss(&self) -> f64 {
        let pit_zone_lap_frac = self.track_length_pit_zone / self.length;
        self.real_length_pit_zone / self.pit_speedlimit
            - (self.t_q + self.t_gap_racepace) * 1.04 * pit_zone_lap_frac
    }
}
