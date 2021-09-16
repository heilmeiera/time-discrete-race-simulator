use crate::pre::read_sim_pars::SimPars;
use crate::pre::sim_opts::SimOpts;
use anyhow::Context;
use approx::ulps_eq;
use helpers::general::InputValueError;

/// check_sim_opts_pars assures that the inserted options and parameters are within reasonable
/// limits and raises an error if not.
pub fn check_sim_opts_pars(sim_opts: &SimOpts, sim_pars: &SimPars) -> anyhow::Result<()> {
    // PART 1: SIMULATION OPTIONS
    if !(0.001 <= sim_opts.timestep_size && sim_opts.timestep_size <= 1.0) {
        return Err(InputValueError).context(format!(
            "timestep_size is {:.3}s, which is not within the reasonable range of [0.001, 1.0]s!",
            sim_opts.timestep_size
        ));
    }

    if sim_opts.no_sim_runs < 1 {
        return Err(InputValueError).context(format!(
            "no_sim_runs must be at least equal to one, but is {}!",
            sim_opts.no_sim_runs
        ));
    }

    if sim_opts.gui && sim_opts.no_sim_runs != 1 {
        return Err(InputValueError)
            .context("If gui is activated, no_sim_runs must be equal to one!");
    }

    if sim_opts.gui && !(0.1 <= sim_opts.realtime_factor && sim_opts.realtime_factor <= 100.0) {
        return Err(InputValueError).context(format!(
            "realtime_factor is {:.3}, which is not within the reasonable range of [0.1, 100.0]!",
            sim_opts.realtime_factor
        ));
    }

    // PART 2: SIMULATION PARAMETERS
    // TRACK ---------------------------------------------------------------------------------------
    if sim_pars.track_pars.s12 <= 0.0 || sim_pars.track_pars.length <= sim_pars.track_pars.s12 {
        return Err(InputValueError)
            .context("s12 is not within the required range (0.0, track_length)!");
    }
    if sim_pars.track_pars.s23 <= 0.0 || sim_pars.track_pars.length <= sim_pars.track_pars.s23 {
        return Err(InputValueError)
            .context("s23 is not within the required range (0.0, track_length)!");
    }
    if sim_pars
        .track_pars
        .drs_measurement_points
        .iter()
        .any(|&s| s < 0.0 || sim_pars.track_pars.length <= s)
    {
        return Err(InputValueError).context(
            "A DRS measurement point is not within the required range [0.0, track_length)!",
        );
    }
    if sim_pars
        .track_pars
        .pit_zone
        .iter()
        .any(|&s| s < 0.0 || sim_pars.track_pars.length <= s)
    {
        return Err(InputValueError).context(
            "Pit zone entry or exit is not within the required range [0.0, track_length)!",
        );
    }
    if sim_pars.track_pars.overtaking_zones.iter().any(|zone| {
        zone.iter()
            .any(|&s| s < 0.0 || sim_pars.track_pars.length <= s)
    }) {
        return Err(InputValueError).context(
            "An overtaking zone entry or exit is not within the required range \
            [0.0, track_length)!",
        );
    }

    // STRATEGY ------------------------------------------------------------------------------------
    for car_pars in sim_pars.car_pars_all.values() {
        if !(car_pars.strategy.len() >= 1) {
            return Err(InputValueError).context(
                "There must be at least one startegy entry that contains the start configuration!",
            );
        }

        if car_pars.strategy[0].inlap != 0
            || car_pars.strategy[0].compound.is_empty()
            || car_pars.strategy[0].driver_initials.is_empty()
            || !ulps_eq!(car_pars.strategy[0].refuel_mass, 0.0)
        {
            return Err(InputValueError).context(format!(
                "The first strategy entry of car {} does not fulfill the requirements (inlap 0, \
                start compound and driver defined, refuel mass 0.0)!",
                car_pars.car_no
            ));
        }

        for i in 1..car_pars.strategy.len() {
            if car_pars.strategy[i].inlap <= car_pars.strategy[i - 1].inlap {
                return Err(InputValueError).context(format!(
                    "The inlap of the {}. strategy entry of car {} is less or equal to that of the \
                    previous entry!",
                    i + 1, car_pars.car_no
                ));
            }
        }
    }

    Ok(())
}
