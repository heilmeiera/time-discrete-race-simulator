use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DegrModel {
    Lin,
    Quad,
    Cub,
    Ln,
}

/// * `degr_model` - Tire degradation model -> lin (linear), quad (quadratic), cub (cubic),
/// ln (logarithmic)
/// * `t_add_coldtires` - (s) Time loss due to cold (i.e. pre-heated) tires
/// * `k_0` - (s) Tire degradation parameter -> offset of the tire compound for fresh tires
/// * `k_1_lin` - (s/lap) Tire degradation parameter (linear model)
/// * `k_1_quad` - (s/lap) Tire degradation parameter (quadratic model)
/// * `k_2_quad` - (s/lap^2) Tire degradation parameter (quadratic model)
/// * `k_1_cub` - (s/lap) Tire degradation parameter (cubic model)
/// * `k_2_cub` - (s/lap^2) Tire degradation parameter (cubic model)
/// * `k_3_cub` - (s/lap^3) Tire degradation parameter (cubic model)
/// * `k_1_ln` - (?) Tire degradation parameter (logarithmic model)
/// * `k_2_ln` - (?) Tire degradation parameter (logarithmic model) -> scaling of age
#[derive(Debug, Deserialize, Clone)]
pub struct DegrPars {
    pub degr_model: DegrModel,
    pub t_add_coldtires: f64,
    pub k_0: f64,
    pub k_1_lin: Option<f64>,
    pub k_1_quad: Option<f64>,
    pub k_2_quad: Option<f64>,
    pub k_1_cub: Option<f64>,
    pub k_2_cub: Option<f64>,
    pub k_3_cub: Option<f64>,
    pub k_1_ln: Option<f64>,
    pub k_2_ln: Option<f64>,
}

#[derive(Debug)]
pub struct Tireset {
    pub compound: String,
    pub age_tot: u32,
    pub age_cur_stint: u32,
}

impl Tireset {
    pub fn new(compound: String, age_tot: u32) -> Tireset {
        Tireset {
            compound,
            age_tot,
            age_cur_stint: 0,
        }
    }

    /// drive_lap increases the tire age by one lap to consider tire degradation.
    pub fn drive_lap(&mut self) {
        self.age_cur_stint += 1;
        self.age_tot += 1;
    }

    /// t_add_tireset returns the current time loss due to tire degradation and possibly cold tires.
    pub fn t_add_tireset(&self, degr_pars: &DegrPars) -> f64 {
        if self.age_cur_stint == 0 {
            self.calc_tire_degr(degr_pars) + degr_pars.t_add_coldtires
        } else {
            self.calc_tire_degr(degr_pars)
        }
    }

    /// calc_tire_degr returns a tire degradation time delta that is calculated according to one of
    /// the following functions:
    ///
    /// * `linear model`: t_tire_degr = k_0 + k_1_lin  * age
    /// * `quadratic model`: t_tire_degr = k_0 + k_1_quad * age + k_2_quad * age**2
    /// * `cubic model`: t_tire_degr = k_0 + k_1_cub  * age + k_2_cub  * age**2 + k_3_cub * age**3
    /// * `logarithmic model`: t_tire_degr = k_0 + k_1_ln   * ln(k_2_ln * age + 1)
    ///
    /// `age` is the total tire age in laps at the start of the current lap.
    fn calc_tire_degr(&self, degr_pars: &DegrPars) -> f64 {
        let age_tot = self.age_tot as f64;

        match degr_pars.degr_model {
            // linear tire degradation model
            DegrModel::Lin => {
                degr_pars.k_0 + degr_pars.k_1_lin.expect("Missing parameter k_1_lin!") * age_tot
            }

            // quadratic tire degradation model
            DegrModel::Quad => {
                degr_pars.k_0
                    + degr_pars.k_1_quad.expect("Missing parameter k_1_quad!") * age_tot
                    + degr_pars.k_2_quad.expect("Missing parameter k_2_quad!") * age_tot.powf(2.0)
            }

            // cubic tire degradation model
            DegrModel::Cub => {
                degr_pars.k_0
                    + degr_pars.k_1_cub.expect("Missing parameter k_1_cub!") * age_tot
                    + degr_pars.k_2_cub.expect("Missing parameter k_2_cub!") * age_tot.powf(2.0)
                    + degr_pars.k_3_cub.expect("Missing parameter k_3_cub!") * age_tot.powf(3.0)
            }

            // logarithmic tire degradation model
            DegrModel::Ln => {
                degr_pars.k_0
                    + degr_pars.k_1_ln.expect("Missing parameter k_1_ln!")
                        * (degr_pars.k_2_ln.expect("Missing parameter k_2_ln!") * age_tot + 1.0)
                            .ln()
            }
        }
    }
}
