use std::fs::File;

use gmt_dos_clients_fem::{Model, Switch};
use gmt_fem::FEM;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut fem = FEM::from_env().unwrap();

    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);

    let sid = 1;
    let vc_f2d = fem
        .switch_inputs_by_name(vec![format!("MC_M2_S{sid}_VC_delta_F")], Switch::On)
        .and_then(|fem| {
            fem.switch_outputs_by_name(vec![format!("MC_M2_S{sid}_VC_delta_D")], Switch::On)
        })
        .map(|fem| {
            fem.reduced_static_gain()
                .unwrap_or_else(|| fem.static_gain())
        })?;

    serde_pickle::to_writer(
        &mut File::create("vc_f2d.pkl")?,
        &vc_f2d,
        Default::default(),
    )?;

    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);

    let fs_f2d = fem
        .switch_inputs_by_name(vec![format!("MC_M2_S{sid}_VC_delta_F")], Switch::On)
        .and_then(|fem| {
            fem.switch_outputs_by_name(vec![format!("M2_segment_{sid}_axial_d")], Switch::On)
        })
        .map(|fem| {
            fem.reduced_static_gain()
                .unwrap_or_else(|| fem.static_gain())
        })?;

    fem.switch_inputs(Switch::On, None)
        .switch_outputs(Switch::On, None);

    serde_pickle::to_writer(
        &mut File::create("fs_f2d.pkl")?,
        &fs_f2d,
        Default::default(),
    )?;

    Ok(())
}
