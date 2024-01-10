use gmt_dos_actors::actorscript;
use gmt_dos_clients::{Logging, Signal, Signals};
use gmt_dos_clients_fem::{DiscreteModalSolver, ExponentialMatrix, Model, Switch};
use gmt_dos_clients_io::gmt_m2::M2PositionerNodes;
use gmt_dos_clients_io::{
    gmt_fem::{
        inputs::MCM2SmHexF,
        outputs::{MCM2Lcl6D, MCM2SmHexD, MCM2RB6D},
    },
    gmt_m2::{M2PositionerForces, M2RigidBodyMotions},
    mount::{MountEncoders, MountSetPoint, MountTorques},
};
use gmt_dos_clients_m2_ctrl::positioner::AsmsPositioners;
use gmt_dos_clients_mount::Mount;
use nalgebra as na;

/*
export FEM_REPO=/home/rconan/mnt/20230131_1605_zen_30_M1_202110_ASM_202208_Mount_202111/
cargo test --release  --package gmt_dos-clients_m2-ctrl --features serde --test mount-asms-positioners -- main --exact --nocapture
 */

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sim_sampling_frequency = 8000;
    let sim_duration = 3_usize; // second
    let n_step = sim_sampling_frequency * sim_duration;

    let mut fem = gmt_fem::FEM::from_env().unwrap();

    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);
    let hex_f2d = {
        let hex_f2d = fem
            .switch_inputs_by_name(vec!["MC_M2_SmHex_F"], Switch::On)
            .and_then(|fem| fem.switch_outputs_by_name(vec!["MC_M2_SmHex_D"], Switch::On))
            .map(|fem| {
                fem.reduced_static_gain()
                    .unwrap_or_else(|| fem.static_gain())
            })?;
        let left = na::DMatrix::from_columns(&hex_f2d.column_iter().step_by(2).collect::<Vec<_>>());
        let right = na::DMatrix::from_columns(
            &hex_f2d.column_iter().skip(1).step_by(2).collect::<Vec<_>>(),
        );
        let hex_f2d = left - right;
        let left = na::DMatrix::from_rows(&hex_f2d.row_iter().step_by(2).collect::<Vec<_>>());
        let right =
            na::DMatrix::from_rows(&hex_f2d.row_iter().skip(1).step_by(2).collect::<Vec<_>>());
        left - right
    };

    fem.switch_inputs(Switch::Off, None)
        .switch_outputs(Switch::Off, None);
    let hex_f_2_rb_d = {
        let hex_f_2_rb_d = fem
            .switch_inputs_by_name(vec!["MC_M2_SmHex_F"], Switch::On)
            .and_then(|fem| fem.switch_outputs_by_name(vec!["MC_M2_RB_6D"], Switch::On))
            .map(|fem| {
                fem.reduced_static_gain()
                    .unwrap_or_else(|| fem.static_gain())
            })?;
        let left =
            na::DMatrix::from_columns(&hex_f_2_rb_d.column_iter().step_by(2).collect::<Vec<_>>());
        let right = na::DMatrix::from_columns(
            &hex_f_2_rb_d
                .column_iter()
                .skip(1)
                .step_by(2)
                .collect::<Vec<_>>(),
        );
        left - right
    };

    let mat = hex_f2d
        * hex_f_2_rb_d
            .try_inverse()
            .expect("failed to inverse the positioners forces to rigid0body-motions matrix");
    let r2p = na::SMatrix::<f64, 42, 42>::from_iterator(mat.into_iter().map(|x| *x));

    fem.switch_inputs(Switch::On, None)
        .switch_outputs(Switch::On, None);
    fem.switch_inputs(Switch::On, None)
        .switch_outputs(Switch::On, None);

    let plant = DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
        .sampling(8e3)
        .proportional_damping(2. / 100.)
        .including_mount()
        .ins::<MCM2SmHexF>()
        .outs::<MCM2SmHexD>()
        .outs::<MCM2Lcl6D>()
        .use_static_gain_compensation()
        .outs::<MCM2RB6D>()
        .build()?;

    let positioners = AsmsPositioners::new(r2p);

    // MOUNT CONTROL
    let mount_setpoint = Signals::new(3, n_step);
    let mount = Mount::new();

    let rbm_fun =
        |i: usize, sid: u8| (-1f64).powi(i as i32) * (1 + (i % 3)) as f64 + sid as f64 / 10_f64;
    let rbm = (1..=7).fold(Signals::new(6 * 7, n_step), |signals_sid, sid| {
        (0..6).fold(signals_sid, |signals, i| {
            signals.channel(
                i + 6 * (sid - 1) as usize,
                Signal::Sigmoid {
                    amplitude: rbm_fun(i, sid) * 1e-6,
                    sampling_frequency_hz: sim_sampling_frequency as f64,
                },
            )
        })
    });

    let plant_logging: Logging<f64> = Logging::<f64>::new(1);

    actorscript! {
        1: mount_setpoint[MountSetPoint] -> mount[MountTorques] -> plant[MountEncoders]! -> mount
        1: rbm[M2RigidBodyMotions] -> positioners[M2PositionerForces] -> plant[M2PositionerNodes]! -> positioners
        1: plant[M2RigidBodyMotions].. -> plant_logging
    }

    let rbm_err = (*plant_logging.lock().await)
        .chunks()
        .last()
        .unwrap()
        .chunks(6)
        .map(|x| x.iter().map(|x| x * 1e6).collect::<Vec<_>>())
        .enumerate()
        .inspect(|(i, x)| println!("{:2}: {:+.1?}", i, x))
        .map(|(i, x)| {
            x.iter()
                .enumerate()
                .map(|(j, x)| x - rbm_fun(j, i as u8 + 1))
                .map(|x| x * x)
                .sum::<f64>()
                / 6f64
        })
        .map(|x| x.sqrt())
        .sum::<f64>()
        / 7f64;
    assert!(dbg!(rbm_err) < 5e-2);

    Ok(())
}
