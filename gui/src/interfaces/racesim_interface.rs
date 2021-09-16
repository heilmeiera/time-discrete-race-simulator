use flume::Receiver;
use racesim::interfaces::gui_interface::RaceState;

#[derive(Debug)]
pub struct RacesimInterface {
    pub rx: Receiver<RaceState>,
    pub race_state: RaceState,
}

impl RacesimInterface {
    pub fn update(&mut self) {
        // loop to obtain the latest race state in the channel
        let mut tmp_message = self.rx.try_recv();
        let mut message = tmp_message.clone();

        while tmp_message.is_ok() {
            message = tmp_message.clone();
            tmp_message = self.rx.try_recv();
        }

        // update data stored in the race interface (those are used within the GUI)
        if let Ok(x) = message {
            self.race_state = x;

            // sort car states by car number to make sure the drawing does not flicker (even though
            // the order should already be ordered from RS side)
            self.race_state
                .car_states
                .sort_by(|a, b| b.car_no.cmp(&a.car_no));
        }
    }
}
