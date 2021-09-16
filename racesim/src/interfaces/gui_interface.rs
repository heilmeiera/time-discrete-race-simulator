use crate::core::race::FlagState;

pub const MAX_GUI_UPDATE_FREQUENCY: f64 = 20.0;

#[derive(Debug, Clone, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Default)]
pub struct CarState {
    pub car_no: u32,
    pub driver_initials: String,
    pub color: RgbColor,
    pub race_prog: f64,
}

#[derive(Debug, Clone, Default)]
pub struct RaceState {
    pub car_states: Vec<CarState>,
    pub flag_state: FlagState,
}
