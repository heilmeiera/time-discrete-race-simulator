use anyhow::Context;
use helpers::general::{lin_interp, InputValueError};
use helpers::geometry::{Point2d, Vector2d};
use serde::Deserialize;
use std::fs::OpenOptions;
use std::path::Path;

#[derive(Debug)]
pub enum ZoneType {
    PitZone,
    OvertakingZone,
}

#[derive(Debug)]
pub struct Zone {
    pub zone_type: ZoneType,
    pub centerline: Vec<Point2d>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CsvTrackEl {
    pub x_m: f64,
    pub y_m: f64,
    pub w_tr_left_m: f64,
    pub w_tr_right_m: f64,
}

#[derive(Debug, Clone)]
pub struct TrackEl {
    pub s: f64,
    pub coords: Point2d,
}

#[derive(Debug)]
pub struct Track {
    pub track_cl: Vec<TrackEl>,
    pub s12: f64,
    pub s23: f64,
    pub drs_measurement_points: Vec<f64>,
    pub pit_zone: [f64; 2],
    pub overtaking_zones: Vec<[f64; 2]>,
    pub clockwise: bool,
}

impl Track {
    pub fn from_csv(
        trackfile_path: &Path,
        track_length: f64,
        s12: f64,
        s23: f64,
        drs_measurement_points: Vec<f64>,
        pit_zone: [f64; 2],
        overtaking_zones: Vec<[f64; 2]>,
    ) -> anyhow::Result<Track> {
        // check input
        if s12 <= 0.0 || track_length <= s12 {
            return Err(InputValueError)
                .context("s12 is not within the required range (0.0, track_length)!");
        }
        if s23 <= 0.0 || track_length <= s23 {
            return Err(InputValueError)
                .context("s23 is not within the required range (0.0, track_length)!");
        }
        if drs_measurement_points
            .iter()
            .any(|&s| s < 0.0 || track_length <= s)
        {
            return Err(InputValueError).context(
                "A DRS measurement point is not within the required range [0.0, track_length)!",
            );
        }
        if pit_zone.iter().any(|&s| s < 0.0 || track_length <= s) {
            return Err(InputValueError).context(
                "Pit zone entry or exit is not within the required range [0.0, track_length)!",
            );
        }
        if overtaking_zones
            .iter()
            .any(|zone| zone.iter().any(|&s| s < 0.0 || track_length <= s))
        {
            return Err(InputValueError).context(
                "An overtaking zone entry or exit is not within the required range \
                [0.0, track_length)!",
            );
        }

        // open file
        let fh = OpenOptions::new()
            .read(true)
            .open(trackfile_path)
            .context(format!(
                "Failed to open track file {}!",
                trackfile_path.to_str().unwrap()
            ))?;

        // read and parse csv track data
        let mut csv_reader = csv::Reader::from_reader(&fh);
        let mut csv_track_cl: Vec<CsvTrackEl> = vec![];

        for result in csv_reader.deserialize() {
            let csv_track_el: CsvTrackEl = result?;
            csv_track_cl.push(csv_track_el);
        }

        // create track and close it
        let mut track_cl: Vec<TrackEl> = csv_track_cl
            .iter()
            .map(|el| TrackEl {
                s: 0.0,
                coords: Point2d {
                    x: el.x_m,
                    y: el.y_m,
                },
            })
            .collect();

        track_cl.push(track_cl[0].clone());

        // calculate curvi-linear distance s up to each element
        for i in 1..track_cl.len() {
            track_cl[i].s = track_cl[i - 1].s
                + track_cl[i]
                    .coords
                    .as_vector2d()
                    .sub(&track_cl[i - 1].coords.as_vector2d())
                    .abs()
        }

        // scale s to fit inserted track length
        let scale_factor = track_length / track_cl.last().unwrap().s;
        for track_el in track_cl.iter_mut() {
            track_el.s *= scale_factor
        }

        // determine if track is driven clockwise or counter-clockwise using the sum of cross
        // products
        let mut tmp_area = 0.0;

        for i in 0..track_cl.len() - 1 {
            let vec_1 = track_cl[i + 1]
                .coords
                .as_vector2d()
                .sub(&track_cl[i].coords.as_vector2d());
            let vec_2 = track_cl[(i + 2) % track_cl.len()]
                .coords
                .as_vector2d()
                .sub(&track_cl[i + 1].coords.as_vector2d());
            tmp_area += vec_1.cross(&vec_2);
        }

        let clockwise = tmp_area > 0.0;

        Ok(Track {
            track_cl,
            s12,
            s23,
            drs_measurement_points,
            pit_zone,
            overtaking_zones,
            clockwise,
        })
    }

    pub fn get_axes_expansion(&self, padding_size: f64) -> [f64; 4] {
        // determine min and max x and y values
        let (mut x_min, mut x_max, mut y_min, mut y_max) = self.track_cl.iter().fold(
            (
                self.track_cl[0].coords.x,
                self.track_cl[0].coords.x,
                self.track_cl[0].coords.y,
                self.track_cl[0].coords.y,
            ),
            |(x_min, x_max, y_min, y_max), track_el| {
                let x_min_tmp = if track_el.coords.x < x_min {
                    track_el.coords.x
                } else {
                    x_min
                };
                let x_max_tmp = if track_el.coords.x > x_max {
                    track_el.coords.x
                } else {
                    x_max
                };
                let y_min_tmp = if track_el.coords.y < y_min {
                    track_el.coords.y
                } else {
                    y_min
                };
                let y_max_tmp = if track_el.coords.y > y_max {
                    track_el.coords.y
                } else {
                    y_max
                };

                (x_min_tmp, x_max_tmp, y_min_tmp, y_max_tmp)
            },
        );

        // apply padding
        x_min -= padding_size;
        x_max += padding_size;
        y_min -= padding_size;
        y_max += padding_size;

        // update min and max values such that its a square shape
        let width = x_max - x_min;
        let height = y_max - y_min;

        if width > height {
            let diff = width - height;
            y_min -= diff / 2.0;
            y_max += diff / 2.0;
        } else {
            let diff = height - width;
            x_min -= diff / 2.0;
            x_max += diff / 2.0;
        }

        [x_min, x_max, y_min, y_max]
    }

    pub fn get_zones(&self) -> Vec<Zone> {
        let mut zones = vec![];

        // pit zone
        let tmp_centerline = self.get_zone_centerline(&self.pit_zone);

        zones.push(Zone {
            zone_type: ZoneType::PitZone,
            centerline: tmp_centerline,
        });

        // overtaking zones
        for overtaking_zone in self.overtaking_zones.iter() {
            let tmp_centerline = self.get_zone_centerline(overtaking_zone);

            zones.push(Zone {
                zone_type: ZoneType::OvertakingZone,
                centerline: tmp_centerline,
            });
        }

        zones
    }

    fn get_zone_centerline(&self, zone: &[f64; 2]) -> Vec<Point2d> {
        // determine start and end index
        let mut start_idx = self.track_cl.iter().position(|el| zone[0] <= el.s).unwrap();

        if start_idx == self.track_cl.len() - 1 {
            start_idx = 0;
        }

        let mut end_idx = self.track_cl.iter().position(|el| zone[1] <= el.s).unwrap();

        if end_idx == 0 {
            end_idx = self.track_cl.len() - 1;
        }

        // get centerline from start to end index (handling the case of a zone that crosses the SF
        // line)
        let mut tmp_centerline: Vec<Point2d> = Vec::new();

        if start_idx < end_idx {
            tmp_centerline.extend((start_idx..end_idx).map(|i| self.track_cl[i].coords.clone()));
        } else {
            tmp_centerline.extend(
                (start_idx..self.track_cl.len() - 1).map(|i| self.track_cl[i].coords.clone()),
            );
            tmp_centerline.extend((0..end_idx).map(|i| self.track_cl[i].coords.clone()));
        }

        tmp_centerline
    }

    pub fn get_dists_for_race_progs(&self, race_progs: &[f64]) -> Vec<f64> {
        // get lap fractions
        let mut lap_fracs: Vec<f64> = race_progs.iter().map(|prog| prog.fract()).collect();

        // normalize input (required for race start)
        for lap_frac in lap_fracs.iter_mut() {
            if *lap_frac < 0.0 {
                *lap_frac += 1.0
            }
        }

        // calculate distance for lap fracs
        lap_fracs
            .iter()
            .map(|frac| frac * self.track_cl.last().unwrap().s)
            .collect()
    }

    pub fn get_coords_for_dists(&self, dists: &[f64]) -> Vec<Point2d> {
        // collect s, x, y for interpolation
        let s: Vec<f64> = self.track_cl.iter().map(|el| el.s).collect();
        let x: Vec<f64> = self.track_cl.iter().map(|el| el.coords.x).collect();
        let y: Vec<f64> = self.track_cl.iter().map(|el| el.coords.y).collect();

        // calculate coordinates for given distances
        let mut coords: Vec<Point2d> = Vec::with_capacity(dists.len());

        for dist in dists.iter() {
            // interpolate exact position
            coords.push(Point2d {
                x: lin_interp(*dist, &s, &x),
                y: lin_interp(*dist, &s, &y),
            })
        }

        coords
    }

    pub fn get_normvecs_for_dists(&self, dists: &[f64]) -> Vec<Vector2d> {
        dists
            .iter()
            .map(|dist| self.calc_normvec_normalized_for_dist(*dist))
            .collect()
    }

    fn calc_normvec_normalized_for_dist(&self, dist: f64) -> Vector2d {
        if dist > self.track_cl.last().unwrap().s {
            panic!("Inserted distance is greater than the track length!")
        }

        // determine idx of first point with a track distance greater than cur_dist
        let tmp_idx = self.track_cl.iter().position(|track_el| dist < track_el.s);

        let idx = match tmp_idx {
            // normal case
            Some(x) => x,
            // this is only the case if dist is equal to the track length -> set index of last
            // element
            _ => self.track_cl.len() - 1,
        };

        // determine normal vector based on the tangent vector between that point and the previous
        // point
        let tan_vec = self.track_cl[idx]
            .coords
            .as_vector2d()
            .sub(&self.track_cl[idx - 1].coords.as_vector2d());

        tan_vec.normalized().normal_vector()
    }
}
