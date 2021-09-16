use crate::core::track::{Track, ZoneType};
use crate::interfaces::racesim_interface::RacesimInterface;
use eframe::{egui, epi};
use flume::Receiver;
use helpers::buffer::RingBuffer;
use helpers::general::max;
use racesim::core::race::RacePars;
use racesim::core::track::TrackPars;
use racesim::interfaces::gui_interface::RaceState;
use std::fmt::Write;
use std::path::Path;
use std::time::Instant;

#[derive(Debug)]
pub struct CarStateGui {
    pub color: egui::Color32,
    pub pos: egui::Pos2,
    pub text_pos: egui::Pos2,
    pub text: String,
}

#[derive(Debug)]
pub struct RaceInfo {
    pub tot_no_laps: u32,
}

#[derive(Debug)]
pub struct RacePlot {
    pub racesim_interface: RacesimInterface,
    pub race_info: RaceInfo,
    pub track: Track,
    pub centerline_cl: Vec<egui::Pos2>,
    pub prev_update: Instant,
    pub prev_update_durations: RingBuffer<u32>,
}

impl RacePlot {
    pub fn new(
        rx: Receiver<RaceState>,
        race_pars: &RacePars,
        track_pars: &TrackPars,
        trackfile_path: &Path,
    ) -> anyhow::Result<RacePlot> {
        // set up interface
        let racesim_interface = RacesimInterface {
            rx,
            race_state: Default::default(),
        };

        // get relevant race information
        let race_info = RaceInfo {
            tot_no_laps: race_pars.tot_no_laps,
        };

        // load track
        let track = Track::from_csv(
            trackfile_path,
            track_pars.length,
            track_pars.s12,
            track_pars.s23,
            track_pars.drs_measurement_points.to_owned(),
            track_pars.pit_zone,
            track_pars.overtaking_zones.to_owned(),
        )?;

        // get centerline from track (saved separately such that this must not be repeated in each
        // call)
        let mut centerline_cl = Vec::with_capacity(track.track_cl.len());

        for track_el in track.track_cl.iter() {
            centerline_cl.push(egui::Pos2 {
                x: track_el.coords.x as f32,
                y: track_el.coords.y as f32,
            })
        }

        // create race plot
        Ok(RacePlot {
            racesim_interface,
            race_info,
            track,
            centerline_cl,
            prev_update: Instant::now(),
            prev_update_durations: RingBuffer::new(10),
        })
    }

    pub fn set_ui_content(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // PREPARATIONS ----------------------------------------------------------------------------
        // get UI handles
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::drag());

        // get transformation from x/y to pixels in the window (y axis must be inverted)
        let [x_min, x_max, y_min, y_max] = self.track.get_axes_expansion(50.0);

        let to_screen = egui::emath::RectTransform::from_to(
            egui::emath::Rect::from_min_max(
                egui::Pos2 {
                    x: x_min as f32,
                    y: y_max as f32,
                },
                egui::Pos2 {
                    x: x_max as f32,
                    y: y_min as f32,
                },
            ),
            response.rect,
        );

        // create vector for drawn shapes
        let mut shapes = vec![];

        // TRACK DRAWING ---------------------------------------------------------------------------
        // add track centerline
        let centerline_cl_tmp: Vec<egui::Pos2> =
            self.centerline_cl.iter().map(|p| to_screen * *p).collect();

        shapes.push(egui::Shape::line(
            centerline_cl_tmp,
            egui::Stroke::new(3.0, egui::Color32::WHITE),
        ));

        // add zones
        let zones = self.track.get_zones();

        for zone in zones.iter() {
            let tmp_centerline: Vec<egui::Pos2> = zone
                .centerline
                .iter()
                .map(|coords| egui::Pos2 {
                    x: coords.x as f32,
                    y: coords.y as f32,
                })
                .collect();
            let tmp_color = if matches!(zone.zone_type, ZoneType::PitZone) {
                // pit zone -> orange
                egui::Color32::from_rgb(255, 128, 0)
            } else {
                // overtaking zone -> blue
                egui::Color32::from_rgb(0, 128, 255)
            };

            shapes.push(egui::Shape::line(
                tmp_centerline.iter().map(|&x| to_screen * x).collect(),
                egui::Stroke::new(7.0, tmp_color),
            ));
        }

        // add track's sector boundaries and DRS measurement points
        let mut tmp_dists = vec![0.0, self.track.s12, self.track.s23];
        let mut tmp_texts = vec![String::from("SF"), String::from("S12"), String::from("S23")];
        for (i, tmp_dist) in self.track.drs_measurement_points.iter().enumerate() {
            tmp_dists.push(*tmp_dist);
            tmp_texts.push(format!("DRSM{}", i + 1));
        }
        let tmp_coords = self.track.get_coords_for_dists(&tmp_dists);
        let tmp_normvecs = self.track.get_normvecs_for_dists(&tmp_dists);
        let tmp_sign = if self.track.clockwise { 1.0 } else { -1.0 };
        let text_offset = 60.0;
        let bound_length = 40.0;

        for (i, tmp_text) in tmp_texts.iter().enumerate() {
            let tmp_p1 = tmp_coords[i]
                .as_vector2d()
                .add(&(tmp_normvecs[i].mult(0.5).mult(tmp_sign).mult(bound_length)))
                .as_point2d();
            let tmp_p2 = tmp_coords[i]
                .as_vector2d()
                .sub(&(tmp_normvecs[i].mult(0.5).mult(tmp_sign).mult(bound_length)))
                .as_point2d();
            let tmp_text_coords = tmp_coords[i]
                .as_vector2d()
                .add(&(tmp_normvecs[i].mult(tmp_sign).mult(text_offset)))
                .as_point2d();

            let tmp_line = vec![
                egui::Pos2 {
                    x: tmp_p1.x as f32,
                    y: tmp_p1.y as f32,
                },
                egui::Pos2 {
                    x: tmp_p2.x as f32,
                    y: tmp_p2.y as f32,
                },
            ];
            let tmp_text_pos = egui::Pos2 {
                x: tmp_text_coords.x as f32,
                y: tmp_text_coords.y as f32,
            };

            shapes.push(egui::Shape::line(
                tmp_line.iter().map(|&x| to_screen * x).collect(),
                egui::Stroke::new(3.0, egui::Color32::WHITE),
            ));
            shapes.push(egui::Shape::text(
                ui.fonts(),
                to_screen * tmp_text_pos,
                egui::Align2::CENTER_CENTER,
                tmp_text,
                egui::TextStyle::Body,
                egui::Color32::WHITE,
            ));
        }

        // CARS DRAWING ----------------------------------------------------------------------------
        // calculate current car coordinates and prepare the GUI car states for drawing
        let tmp_race_progs: Vec<f64> = self
            .racesim_interface
            .race_state
            .car_states
            .iter()
            .map(|car_state| car_state.race_prog)
            .collect();
        let tmp_dists = self.track.get_dists_for_race_progs(&tmp_race_progs);
        let tmp_coords = self.track.get_coords_for_dists(&tmp_dists);
        let tmp_normvecs = self.track.get_normvecs_for_dists(&tmp_dists);
        let tmp_sign = if self.track.clockwise { -1.0 } else { 1.0 };
        let text_offset = 100.0;

        let mut car_states_gui: Vec<CarStateGui> =
            Vec::with_capacity(self.racesim_interface.race_state.car_states.len());

        for (i, car_state) in self
            .racesim_interface
            .race_state
            .car_states
            .iter()
            .enumerate()
        {
            let tmp_text_coords = tmp_coords[i]
                .as_vector2d()
                .add(&(tmp_normvecs[i].mult(text_offset).mult(tmp_sign)))
                .as_point2d();
            let tmp_text = format!("{} ({})", car_state.car_no, car_state.driver_initials);

            let car_state_gui = CarStateGui {
                color: egui::Color32::from_rgb(
                    car_state.color.r,
                    car_state.color.g,
                    car_state.color.b,
                ),
                pos: egui::Pos2 {
                    x: tmp_coords[i].x as f32,
                    y: tmp_coords[i].y as f32,
                },
                text_pos: egui::Pos2 {
                    x: tmp_text_coords.x as f32,
                    y: tmp_text_coords.y as f32,
                },
                text: tmp_text,
            };

            car_states_gui.push(car_state_gui);
        }

        // add car points
        for car_state_gui in car_states_gui.iter() {
            shapes.push(egui::Shape::circle_filled(
                to_screen * car_state_gui.pos,
                7.0,
                car_state_gui.color,
            ));

            shapes.push(egui::Shape::text(
                ui.fonts(),
                to_screen * car_state_gui.text_pos,
                egui::Align2::CENTER_CENTER,
                &car_state_gui.text,
                egui::TextStyle::Body,
                car_state_gui.color,
            ));
        }

        // UPDATE GENERAL INFORMATION TEXT IN GUI --------------------------------------------------
        // add current lap
        let race_progs: Vec<f64> = self
            .racesim_interface
            .race_state
            .car_states
            .iter()
            .map(|car_state| car_state.race_prog)
            .collect();
        let cur_lap_leader = max(&race_progs).trunc() as u32 + 1;
        let mut gen_info_text = format!("Lap: {}/{}\n", cur_lap_leader, self.race_info.tot_no_laps);

        // add flag state
        writeln!(
            &mut gen_info_text,
            "Flag state: {:?}",
            self.racesim_interface.race_state.flag_state
        )
        .unwrap();

        // calculate current UI update duration, append it to the buffer, and set update time
        self.prev_update_durations
            .push(self.prev_update.elapsed().as_millis() as u32);
        self.prev_update = Instant::now();

        // add update frequency
        write!(
            &mut gen_info_text,
            "GUI update frequency: {:.0} Hz",
            1000.0 / self.prev_update_durations.get_avg().unwrap()
        )
        .unwrap();

        // show general informations text in the GUI
        shapes.push(egui::Shape::text(
            ui.fonts(),
            to_screen
                * egui::Pos2 {
                    x: x_min as f32,
                    y: y_max as f32,
                },
            egui::Align2::LEFT_TOP,
            &gen_info_text,
            egui::TextStyle::Body,
            egui::Color32::WHITE,
        ));

        // DRAWING ---------------------------------------------------------------------------------
        // update shapes in UI painter and return response
        painter.extend(shapes);
        response
    }
}

impl epi::App for RacePlot {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame) {
        // update race interface
        self.racesim_interface.update();

        // update UI content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                self.set_ui_content(ui);
            });
        });

        // request repaint of the UI
        ctx.request_repaint();
    }

    fn name(&self) -> &str {
        "Race Plot"
    }
}
