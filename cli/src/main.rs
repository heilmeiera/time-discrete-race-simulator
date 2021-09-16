use clap::Clap;
use flume;
use gui::core::gui::RacePlot;
use racesim::post::race_result::RaceResult;
use racesim::pre::check_sim_opts_pars::check_sim_opts_pars;
use racesim::pre::read_sim_pars::read_sim_pars;
use racesim::pre::sim_opts::SimOpts;
use rayon::prelude::*;
use std::cmp::min;
use std::thread;
use std::time::Instant;

// set maximum number of concurrently running jobs in case of running more than a single simulation
const MAX_NO_CONCURRENT_JOBS: u32 = 200;

fn main() -> anyhow::Result<()> {
    // PRE-PROCESSING ------------------------------------------------------------------------------
    // get simulation options from the command line arguments and read simulation parameters
    let sim_opts: SimOpts = SimOpts::parse();
    let sim_pars = read_sim_pars(sim_opts.parfile_path.as_path())?;

    // check simulation options and parameters
    check_sim_opts_pars(&sim_opts, &sim_pars)?;

    // create vector for the race result and simulate race(s)
    let mut race_results: Vec<RaceResult> = Vec::with_capacity(sim_opts.no_sim_runs as usize);

    // print race details
    println!(
        "INFO: Simulating {} {} with a time step size of {:.3}s",
        sim_pars.track_pars.name, sim_pars.race_pars.season, sim_opts.timestep_size
    );

    // EXECUTION -----------------------------------------------------------------------------------
    if !sim_opts.gui {
        // NON-GUI CASE ----------------------------------------------------------------------------
        let t_start = Instant::now();

        if sim_opts.no_sim_runs == 1 {
            // SINGLE THREAD -----------------------------------------------------------------------
            race_results.push(
                racesim::core::handle_race::handle_race(
                    &sim_pars,
                    sim_opts.timestep_size,
                    sim_opts.debug,
                    None,
                    1.0,
                )
                .unwrap(),
            );
        } else {
            // MULTIPLE THREADS --------------------------------------------------------------------
            let mut no_races_left = sim_opts.no_sim_runs;

            while no_races_left > 0 {
                // calculate number of simulation runs to execute in current loop
                let tmp_no_sim_runs = min(no_races_left, MAX_NO_CONCURRENT_JOBS);

                // simulate the races and save the results
                race_results.par_extend((0..tmp_no_sim_runs).into_par_iter().map(|_| {
                    racesim::core::handle_race::handle_race(
                        &sim_pars,
                        sim_opts.timestep_size,
                        false,
                        None,
                        1.0,
                    )
                    .unwrap()
                }));

                // reduce remaining simulation runs
                no_races_left -= tmp_no_sim_runs;
            }
        }

        println!(
            "INFO: Execution time (total): {}ms",
            t_start.elapsed().as_millis()
        );
    } else {
        // GUI CASE --------------------------------------------------------------------------------
        // create channel for communication between GUI and RS
        let (tx, rx) = flume::unbounded();

        // create a separate thread for the RS (executed in real-time) -> sim_opts and sim_pars get
        // moved and must therefore be copied to be still available afterwards
        let sim_opts_thread = sim_opts.clone();
        let sim_pars_thread = sim_pars.clone();

        let _ = thread::spawn(move || {
            racesim::core::handle_race::handle_race(
                &sim_pars_thread,
                sim_opts_thread.timestep_size,
                sim_opts_thread.debug,
                Some(&tx),
                sim_opts_thread.realtime_factor,
            )
        });

        // start GUI (must be done in the main thread)
        let mut trackfile_path = sim_opts.parfile_path.to_owned();
        trackfile_path.pop();
        trackfile_path.pop();
        trackfile_path.push("tracks");
        trackfile_path.push(&sim_pars.track_pars.name);
        trackfile_path.set_extension("csv");

        let gui = RacePlot::new(
            rx,
            &sim_pars.race_pars,
            &sim_pars.track_pars,
            trackfile_path.as_path(),
        )?;
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(Box::new(gui), native_options);
    }

    // POST-PROCESSING -----------------------------------------------------------------------------
    // print results
    if race_results.len() == 1 {
        race_results[0].print_lap_and_race_times();
    } else {
        // TODO IMPLEMENTATION MISSING
    }

    Ok(())
}
