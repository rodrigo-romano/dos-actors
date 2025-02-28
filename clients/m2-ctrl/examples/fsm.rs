use std::error::Error;

use gmt_dos_clients_fem::{solvers::ExponentialMatrix, DiscreteModalSolver, Model, Switch};
use gmt_dos_clients_io::{
    gmt_fem::{inputs::MCM2PZTF, outputs::MCM2PZTD},
    gmt_m2::fsm::{M2FSMCommand, M2FSMPiezoForces, M2FSMPiezoNodes},
};
use interface::{Read, Update, Write};
use nalgebra as na;

use gmt_dos_clients_m2_ctrl::FsmSegmentInnerController;

pub fn main() -> Result<(), Box<dyn Error>> {
    const SID: u8 = 1;
    let mut fem = gmt_fem::FEM::from_env()?;
    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);
    let pzt_f2d = {
        let pzt_f2d = fem
            .switch_inputs_by_name(vec!["MC_M2_PZT_F"], Switch::On)
            .and_then(|fem| fem.switch_outputs_by_name(vec!["MC_M2_PZT_D"], Switch::On))
            .map(|fem| fem.reduced_static_gain().unwrap())?;
        let left = na::DMatrix::from_columns(&pzt_f2d.column_iter().step_by(2).collect::<Vec<_>>());
        let right = na::DMatrix::from_columns(
            &pzt_f2d.column_iter().skip(1).step_by(2).collect::<Vec<_>>(),
        );
        let pzt_f2d = left - right;
        let left = na::DMatrix::from_rows(&pzt_f2d.row_iter().step_by(2).collect::<Vec<_>>());
        let right =
            na::DMatrix::from_rows(&pzt_f2d.row_iter().skip(1).step_by(2).collect::<Vec<_>>());
        let i = (SID as usize - 1) * 3;
        (left - right).view((i, i), (3, 3)).into_owned()
    };
    println!("{:?}", pzt_f2d.shape());
    let pzt_d2f = pzt_f2d.try_inverse().unwrap();
    let pzt_cmd_p = vec![1e-6, 0., 0.];
    // let pzt_cmd_f = pzt_d2f * na::DVector::from_column_slice(&pzt_cmd_p);
    // dbg!(&pzt_cmd_f);

    let mut forces = vec![vec![0f64; 6]; 7];

    type PLANT = DiscreteModalSolver<ExponentialMatrix>;
    let mut plant = PLANT::from_fem(fem)
        .sampling(1e3)
        .proportional_damping(2. / 100.)
        .ins::<MCM2PZTF>()
        .outs::<MCM2PZTD>()
        .use_static_gain_compensation()
        // .outs::<MCM2RB6D>()
        .build()?;

    type CTRLR = FsmSegmentInnerController<SID>;
    let mut ctrlr = CTRLR::new();

    let mut data = vec![];

    for _ in 0..100 {
        let pzt_d = <PLANT as Write<M2FSMPiezoNodes>>::write(&mut plant).unwrap();

        let diff_d: Vec<_> = pzt_d
            .chunks(6)
            .nth(SID as usize - 1)
            .unwrap()
            .chunks(2)
            .map(|x| x[1] - x[0])
            .collect();

        let cmd_err: Vec<_> = pzt_cmd_p.iter().zip(&diff_d).map(|(x, y)| x - y).collect();
        dbg!(&cmd_err);
        <CTRLR as Read<M2FSMPiezoNodes>>::read(&mut ctrlr, pzt_d.into());
        <CTRLR as Read<M2FSMCommand>>::read(&mut ctrlr, pzt_cmd_p.clone().into());
        ctrlr.update();
        let seg_forces = <CTRLR as Write<M2FSMPiezoForces>>::write(&mut ctrlr).unwrap();
        // dbg!(&seg_forces);
        forces[SID as usize - 1] = seg_forces.as_slice().to_vec();
        <PLANT as Read<M2FSMPiezoForces>>::read(
            &mut plant,
            forces.iter().cloned().flatten().collect::<Vec<_>>().into(),
        );
        plant.update();

        data.push(diff_d);
    }
    {
        let _: complot::Plot = (
            data.into_iter()
                .enumerate()
                .map(|(i, data)| (i as f64 * 1e-3, data)),
            None,
        )
            .into();
    }
    let _: complot::Plot = (
        (0..100).map(|k| {
            let o = 5. * std::f64::consts::PI * k as f64 / 100.;
            let (s, c) = o.sin_cos();
            (o, vec![s, c])
        }),
        complot::complot!("sin_cos.png", xlabel = "x label", ylabel = "y label"),
    )
        .into();
    Ok(())
}
