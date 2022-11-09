use arrow::{Arrow, Get};
use parse_monitors::cfd;
use std::{fs::File, path::Path};

fn main() -> anyhow::Result<()> {
    let mut pssn_fwhm: Vec<Vec<Vec<f64>>> = Vec::with_capacity(60);
    for cfd_case in cfd::Baseline::<2021>::default() {
        println!("CFD CASE: {cfd_case}");
        let mut data = Arrow::from_parquet(
            Path::new(&cfd_case.to_string()).join("pssn-fwhm_domeseeing_free-atmosphere"),
        )?;
        pssn_fwhm.push(data.get("PSSnFwhm")?);
    }
    let pssn_fwhm: Vec<_> = pssn_fwhm.into_iter().map(|x| (x[0][0], x[0][1])).collect();
    serde_pickle::to_writer(
        &mut File::create("PSSnFwhmDomeSeeing_free-atmosphere.pkl")?,
        &pssn_fwhm,
        Default::default(),
    )?;

    Ok(())
}
