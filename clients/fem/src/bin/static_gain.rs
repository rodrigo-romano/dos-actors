use std::fs::File;

use gmt_dos_clients_fem::{Model, Switch};
use gmt_fem::FEM;

fn main() -> anyhow::Result<()> {
    let q: nalgebra::DMatrix<f64> =
        serde_pickle::from_reader(File::open("q.pkl")?, Default::default())?;
    println!("{q}");

    let mut fem = FEM::from_env()?;

    /*     let inputs = vec!["OSS_Harpoint_delta_F".to_string()];
    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);
    let k1 = fem
        .switch_inputs_by_name(inputs.clone(), Switch::On)
        .and_then(|fem| fem.switch_outputs_by_name(vec!["OSS_M1_lcl"], Switch::On))
        .map(|fem| fem.reduced_static_gain().unwrap())?;
    serde_pickle::to_writer(&mut File::create("k1.pkl")?, &k1, Default::default())?;

    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);
    let k2 = fem
        .switch_inputs_by_name(inputs, Switch::On)
        .and_then(|fem| fem.switch_outputs_by_name(vec!["OSS_M1_edge_sensors"], Switch::On))
        .map(|fem| fem.reduced_static_gain().unwrap())?;
    serde_pickle::to_writer(&mut File::create("k2.pkl")?, &k2, Default::default())?; */

    for i in 1..=7 {
        fem.switch_inputs(Switch::Off, None)
            .switch_outputs(Switch::Off, None);
        let mat = fem
            .switch_inputs_by_name(vec![format!("MC_M2_S{i}_VC_delta_F")], Switch::On)
            .and_then(|fem| {
                fem.switch_outputs_by_name(vec![format!("MC_M2_S{i}_VC_delta_D")], Switch::On)
            })
            .map(|fem| fem.reduced_static_gain().unwrap())?;
        serde_pickle::to_writer(
            &mut File::create(format!("G_vc_f2d_{i}.pkl"))?,
            &mat,
            Default::default(),
        )?;
        fem.switch_inputs(Switch::Off, None)
            .switch_outputs(Switch::Off, None);
        let mat = fem
            .switch_inputs_by_name(vec![format!("MC_M2_S{i}_VC_delta_F")], Switch::On)
            .and_then(|fem| fem.switch_outputs_by_name(vec!["MC_M2_lcl_6D"], Switch::On))
            .map(|fem| fem.reduced_static_gain().unwrap())?;
        serde_pickle::to_writer(
            &mut File::create(format!("G_vcf2rbm_{i}.pkl"))?,
            &mat,
            Default::default(),
        )?;
    }

    Ok(())
}

/*
# M1 edge sensor interaction matrix:
import numpy as np
from scipy.io import loadmat
data = np.load("k1.pkl",allow_pickle=True)
k1 = np.asarray(data[0],order="F").reshape(data[2],data[1]).T
data = np.load("k2.pkl",allow_pickle=True)
k2 = np.asarray(data[0],order="F").reshape(data[2],data[1]).T
data = loadmat("M1_edge_sensor_conversion.mat")
A1 = data['A1']
k = A1 @ np.linalg.lstsq(k1.T,k2.T,rcond=None)[0].T
 */
