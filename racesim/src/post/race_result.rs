use std::fmt::Write;

/// CarDriverPair is used to store car number and driver initials for post-processing the results.
pub struct CarDriverPair {
    pub car_no: u32,
    pub driver_initials: String,
}

/// RaceResult contains all race information that is required for post-processing the results.
pub struct RaceResult {
    pub tot_no_laps: u32,
    pub car_driver_pairs: Vec<CarDriverPair>,
    pub laptimes: Vec<Vec<f64>>,
    pub racetimes: Vec<Vec<f64>>,
}

impl RaceResult {
    /// print_lap_and_race_times prints the resulting lap and race times to the console output.
    pub fn print_lap_and_race_times(&self) {
        // create string for lap times and race times
        let mut tmp_string_laptime = String::new();
        let mut tmp_string_racetime = String::new();

        for lap in 1..self.tot_no_laps as usize + 1 {
            write!(&mut tmp_string_laptime, "{:3}, ", lap).unwrap();
            write!(&mut tmp_string_racetime, "{:3}, ", lap).unwrap();

            for i in 0..self.car_driver_pairs.len() {
                if i < self.car_driver_pairs.len() - 1 {
                    write!(&mut tmp_string_laptime, "{:8.3}s, ", self.laptimes[i][lap]).unwrap();
                    write!(
                        &mut tmp_string_racetime,
                        "{:8.3}s, ",
                        self.racetimes[i][lap]
                    )
                    .unwrap();
                } else {
                    writeln!(&mut tmp_string_laptime, "{:8.3}s", self.laptimes[i][lap]).unwrap();
                    writeln!(&mut tmp_string_racetime, "{:8.3}s", self.racetimes[i][lap]).unwrap();
                }
            }
        }

        // create string with car and driver info
        let mut tmp_string_car_driver_info = String::from("lap, ");

        for (i, car_driver_pair) in self.car_driver_pairs.iter().enumerate() {
            if i < self.car_driver_pairs.len() - 1 {
                write!(
                    &mut tmp_string_car_driver_info,
                    "{:3} ({}), ",
                    car_driver_pair.car_no, car_driver_pair.driver_initials
                )
                .unwrap()
            } else {
                write!(
                    &mut tmp_string_car_driver_info,
                    "{:3} ({})",
                    car_driver_pair.car_no, car_driver_pair.driver_initials
                )
                .unwrap()
            }
        }

        // print everything to the console
        println!("RESULT: Lap times");
        println!("{}", tmp_string_car_driver_info);
        println!("{}", tmp_string_laptime);

        println!("RESULT: Race times");
        println!("{}", tmp_string_car_driver_info);
        println!("{}", tmp_string_racetime);
    }
}
