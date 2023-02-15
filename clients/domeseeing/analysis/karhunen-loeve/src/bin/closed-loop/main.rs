use arrow::{Arrow, FileFormat};
use domeseeing::{DomeSeeing, DomeSeeingOpd};
use dos_actors::{clients::Integrator, prelude::*};
use karhunen_loeve::{
    KarhunenLoeve, KarhunenLoeveCoefficients, KarhunenLoeveResidualCoefficients, ResidualOpd, Std,
};
use parse_monitors::cfd;
use vec_box::vec_box;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CFD CASE
    let cfd_case = cfd::Baseline::<2021>::default()
        .into_iter()
        .nth(25)
        .unwrap();
    println!("CFD case: {}", cfd_case);
    // DOME SEEING
    let path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
    let n_sample = 1000;
    let dome_seeing = DomeSeeing::new(path.to_str().unwrap(), 1, Some(n_sample))
        .unwrap()
        .masked();
    let mask = dome_seeing.get(0).expect("failed to retrieve OPD").mask;
    let mut ds: Initiator<_> = dome_seeing.into();
    // OPD STANDART DEVIATION
    let mut opd_std: Terminator<_> = Std::new().into();
    // KARHUNEN-LOEVE BASIS
    let n_mode = 100;
    let mut kl: Actor<_> = KarhunenLoeve::new(n_mode, Some(mask)).into();
    // INTEGRATOR
    let mut integrator: Actor<_> = Integrator::<KarhunenLoeveResidualCoefficients>::new(n_mode)
        .gain(0.)
        .into();
    // ARROW
    let mut logs: Terminator<_> = Arrow::builder(n_mode * n_sample)
        .filename("kl-coefs.mat")
        .file_format(FileFormat::Matlab(Default::default()))
        .build()
        .into();

    // LINKING
    ds.add_output().build::<DomeSeeingOpd>().into_input(&mut kl);
    kl.add_output()
        .multiplex(2)
        .build::<KarhunenLoeveResidualCoefficients>()
        .into_input(&mut integrator)
        .logn(&mut logs, n_mode)
        .await
        .confirm()?;
    integrator
        .add_output()
        .bootstrap()
        .build::<KarhunenLoeveCoefficients>()
        .into_input(&mut kl);
    kl.add_output()
        .build::<ResidualOpd>()
        .into_input(&mut opd_std);
    // MODEL
    Model::new(vec_box![ds, kl, integrator, opd_std, logs])
        .flowchart()
        .check()?
        .run()
        .await?;

    Ok(())
}
