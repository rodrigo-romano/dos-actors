use crseo::{calibrations, Builder, Calibration, Geometric, ATMOSPHERE, GMT, SH24};
use dos_actors::{
    clients::{
        arrow_client::Arrow,
        ceo,
        fsm::*,
        m1::*,
        mount::{Mount, MountEncoders, MountSetPoint, MountTorques},
        windloads,
        windloads::{WindLoads::*, CS},
    },
    prelude::*,
};
use fem::{
    dos::{DiscreteModalSolver, ExponentialMatrix},
    fem_io::*,
    FEM,
};
use lom::{Loader, LoaderTrait, OpticalMetrics, OpticalSensitivities, OpticalSensitivity, LOM};
use nalgebra as na;
use parse_monitors::cfd;
use skyangle::Conversion;
use std::{fs::File, time::Instant};

fn fig_2_mode(sid: u32) -> na::DMatrix<f64> {
    let fig_2_mode: Vec<f64> =
        bincode::deserialize_from(File::open(format!("m1s{sid}fig2mode.bin")).unwrap()).unwrap();
    if sid < 7 {
        na::DMatrix::from_vec(162, 602, fig_2_mode)
    } else {
        na::DMatrix::from_vec(151, 579, fig_2_mode).insert_rows(151, 11, 0f64)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sim_sampling_frequency = 1000_usize;
    let sim_duration = 30f64;
    const CFD_RATE: usize = 1;
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE;

    let loads = vec![
        TopEnd,
        M2Baffle,
        Trusses,
        M1Baffle,
        MirrorCovers,
        LaserGuideStars,
        CRings,
        GIR,
        Platforms,
    ];

    let mut fem = FEM::from_env()?.static_from_env()?;
    let n_io = (fem.n_inputs(), fem.n_outputs());
    println!("{}", fem);
    fem.filter_inputs_by(&[0], |x| {
        loads
            .iter()
            .flat_map(|x| x.fem())
            .fold(false, |b, p| b || x.descriptions.contains(&p))
    });
    let locations: Vec<CS> = fem.inputs[0]
        .as_ref()
        .unwrap()
        .get_by(|x| Some(CS::OSS(x.properties.location.as_ref().unwrap().clone())))
        .into_iter()
        .step_by(6)
        .collect();
    println!("{}", fem);

    let state_space = {
        DiscreteModalSolver::<ExponentialMatrix>::from_fem(fem)
            .sampling(sim_sampling_frequency as f64)
            .proportional_damping(2. / 100.)
            //.truncate_hankel_singular_values(1e-5)
            .max_eigen_frequency(75.)
            .use_static_gain_compensation(n_io)
            .ins::<CFD2021106F>()
            .ins::<OSSElDriveTorque>()
            .ins::<OSSAzDriveTorque>()
            .ins::<OSSRotDriveTorque>()
            .ins::<OSSHarpointDeltaF>()
            .ins::<M1ActuatorsSegment1>()
            .ins::<M1ActuatorsSegment2>()
            .ins::<M1ActuatorsSegment3>()
            .ins::<M1ActuatorsSegment4>()
            .ins::<M1ActuatorsSegment5>()
            .ins::<M1ActuatorsSegment6>()
            .ins::<M1ActuatorsSegment7>()
            .ins::<MCM2SmHexF>()
            .ins::<MCM2PZTF>()
            .outs::<OSSAzEncoderAngle>()
            .outs::<OSSElEncoderAngle>()
            .outs::<OSSRotEncoderAngle>()
            .outs::<OSSHardpointD>()
            .outs::<OSSM1Lcl>()
            .outs::<MCM2Lcl6D>()
            .outs_with::<M1Segment1AxialD>(fig_2_mode(1))
            .outs_with::<M1Segment2AxialD>(fig_2_mode(2))
            .outs_with::<M1Segment3AxialD>(fig_2_mode(3))
            .outs_with::<M1Segment4AxialD>(fig_2_mode(4))
            .outs_with::<M1Segment5AxialD>(fig_2_mode(5))
            .outs_with::<M1Segment6AxialD>(fig_2_mode(6))
            .outs_with::<M1Segment7AxialD>(fig_2_mode(7))
            .outs::<MCM2SmHexD>()
            .outs::<MCM2PZTD>()
            .build()?
    }
    .into_arcx();
    println!("{}", *state_space.lock().await);
    //println!("Y sizes: {:?}", state_space.y_sizes);

    let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
    println!("CFD CASE ({}Hz): {}", cfd_sampling_frequency, cfd_case);
    let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

    let cfd_loads = windloads::CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
        .duration(sim_duration as f64)
        //.time_range((200f64, 340f64))
        .nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
        .m1_segments()
        .m2_segments()
        .build()
        .unwrap()
        .into_arcx();
    let n_step = (sim_duration * sim_sampling_frequency as f64) as usize;
    let logging = Arrow::builder(n_step)
        .entry::<f64, OSSM1Lcl>(42)
        .entry::<f64, MCM2Lcl6D>(42)
        .build()
        .into_arcx();
    let mnt_ctrl = Mount::new().into_arcx();

    (*cfd_loads.lock().await).stop_after(5 * sim_sampling_frequency);

    let model_1 = {
        let mut source: Initiator<_> = Actor::new(cfd_loads.clone());
        let mut sink = Terminator::<_>::new(logging.clone());
        // FEM
        let mut fem: Actor<_> = Actor::new(state_space.clone());
        // MOUNT
        let mut mount: Actor<_> = Actor::new(mnt_ctrl.clone());

        type D = Vec<f64>;

        source
            .add_output()
            .build::<D, CFD2021106F>()
            .into_input(&mut fem);
        source
            .add_output()
            .build::<D, OSSM1Lcl6F>()
            .into_input(&mut fem);
        source
            .add_output()
            .build::<D, MCM2LclForce6F>()
            .into_input(&mut fem);

        let mut mount_set_point: Initiator<_> = Signals::new(3, n_step).into();
        mount_set_point
            .add_output()
            .build::<D, MountSetPoint>()
            .into_input(&mut mount);
        mount
            .add_output()
            .build::<D, MountTorques>()
            .into_input(&mut fem);

        fem.add_output()
            .bootstrap()
            .build::<D, MountEncoders>()
            .into_input(&mut mount);
        fem.add_output()
            .build::<D, OSSM1Lcl>()
            .into_input(&mut sink);
        fem.add_output()
            .build::<D, MCM2Lcl6D>()
            .into_input(&mut sink);

        Model::new(vec![
            Box::new(source),
            Box::new(mount_set_point),
            Box::new(fem),
            Box::new(mount),
            Box::new(sink),
        ])
        .flowchart()
        .check()?
        .run()
    };

    {
        let mut source: Initiator<_> = Actor::new(cfd_loads.clone());
        let mut sink = Terminator::<_>::new(logging.clone());
        // FEM
        let mut fem: Actor<_> = Actor::new(state_space.clone());
        // MOUNT
        let mut mount: Actor<_> = Actor::new(mnt_ctrl.clone());

        type D = Vec<f64>;
    }

    let lom = LOM::builder()
        .rigid_body_motions_record((*logging.lock().await).record()?)?
        .build()?;
    let segment_tiptilt = lom.segment_tiptilt();

    let tau = (sim_sampling_frequency as f64).recip();
    let _: complot::Plot = ((
        segment_tiptilt
            .items()
            .enumerate()
            .map(|(i, data)| (i as f64 * tau, data.to_owned().to_mas())),
        None,
    ))
        .into();

    Ok(())
}
