use dos_actors::{
    clients::{arrow_client::Arrow, windloads},
    prelude::*,
};
use fem::{fem_io::*, FEM};
use parse_monitors::cfd;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pwd = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR")?)
        .join("examples")
        .join("wind_loads");
    std::env::set_var("DATA_REPO", &pwd);

    let sim_sampling_frequency = 1000_usize;

    let sim_duration = 3f64;
    log::info!("Simulation duration: {:6.3}s", sim_duration);

    let cfd_loads = {
        use dos_actors::clients::windloads::WindLoads::*;
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
        let mut fem = FEM::from_env()?;
        println!("{}", fem);
        //println!("{}", fem);
        let cfd_case = cfd::CfdCase::<2021>::colloquial(30, 0, "os", 7)?;
        println!("CFD CASE (20Hz): {}", cfd_case);
        let cfd_path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());

        windloads::CfdLoads::foh(cfd_path.to_str().unwrap(), sim_sampling_frequency)
            .duration(sim_duration as f64)
            //.time_range((200f64, 340f64))
            //.nodes(loads.iter().flat_map(|x| x.keys()).collect(), locations)
            .loads(loads, &mut fem, 0)
            .m1_segments()
            .m2_segments()
            .build()
            .unwrap()
            .into_arcx()
    };
    println!("{}", *cfd_loads.lock().await);

    let n_step = (sim_duration * sim_sampling_frequency as f64) as usize;
    let logging = Arrow::builder(n_step).build().into_arcx();

    let mut source: Initiator<_> = Actor::new(cfd_loads.clone());
    let mut sink = Terminator::<_>::new(logging.clone());

    source
        .add_output()
        .build::<CFD2021106F>()
        .log(&mut sink)
        .await;
    source
        .add_output()
        .build::<OSSM1Lcl6F>()
        .log(&mut sink)
        .await;
    source
        .add_output()
        .build::<MCM2LclForce6F>()
        .log(&mut sink)
        .await;

    Model::new(vec![Box::new(source), Box::new(sink)])
        .name("wind_loads")
        .flowchart()
        .check()?
        .run()
        .wait()
        .await?;

    Ok(())
}
