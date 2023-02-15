use crseo::FromBuilder;
use gmt_dos_clients_crseo::{OpticalModel, OpticalModelOptions, PSSnOptions};
use std::{fs::File, io::Write};

fn main() -> anyhow::Result<()> {
    let gomb = OpticalModel::builder().options(vec![
        /*         OpticalModelOptions::Atmosphere {
            builder: atm,
            time_step: (atm_sampling_frequency as f64).recip(),
        }, */
        OpticalModelOptions::PSSn(PSSnOptions::AtmosphereTelescope(crseo::PSSn::builder())),
    ]);
    let toml = toml::to_string(&gomb).unwrap();
    let mut file = File::create("optical_model.toml")?;
    write!(file, "{}", toml)?;

    Ok(())
}
