use std::fs::File;

use domeseeing_analysis::Opds;
use parse_monitors::cfd;

fn std(data: &[&f64]) -> f64 {
    let (mut sum_squared, mut sum) =
        data.iter()
            .fold((0f64, 0f64), |(mut sum_squared, mut sum), &o| {
                sum_squared += o * o;
                sum += o;
                (sum_squared, sum)
            });
    let n = data.len() as f64;
    sum_squared /= n;
    sum /= n;
    (sum_squared - sum * sum).sqrt()
}

fn main() -> anyhow::Result<()> {
    let n_px = 104;
    let n_px2 = n_px * n_px;
    let n_mode = 1000;
    let kl: Vec<f64> =
        serde_pickle::from_reader(File::open("domeseeing-kl.pkl")?, Default::default())?;

    let cfd_case = cfd::Baseline::<2021>::default()
        .into_iter()
        .nth(25)
        .unwrap();
    println!("CFD case: {}", cfd_case);
    let path = cfd::Baseline::<2021>::path().join(cfd_case.to_string());
    let opds: Opds = bincode::deserialize_from(File::open(path.join("opds.bin")).unwrap()).unwrap();

    let merged_mask: Vec<_> = kl
        .iter()
        .zip(&opds.mask)
        .map(|(k, &m)| !k.is_nan() && m)
        .collect();
    let n_mask: usize = opds.mask.iter().filter_map(|&m| m.then_some(1)).sum();
    let n_merged_mask: usize = merged_mask.iter().filter_map(|&m| m.then_some(1)).sum();
    println!("opd mask/kl+opd mask: {}/{}", n_mask, n_merged_mask);

    let kl_on_mask: Vec<_> = kl
        .chunks(n_px2)
        .flat_map(|kl| {
            kl.iter()
                .zip(&merged_mask)
                .filter_map(|(k, &m)| m.then_some(*k))
        })
        .collect();
    assert_eq!(kl_on_mask.iter().find(|x| x.is_nan()), None);
    assert_eq!(kl_on_mask.len(), n_merged_mask * n_mode);

    let kl_on_mask_orthonorm = zernike::gram_schmidt(kl_on_mask.as_slice(), n_mode);
    println!("KL {}", kl_on_mask_orthonorm.len());

    let a_kl = kl_on_mask_orthonorm.chunks(n_merged_mask).nth(5).unwrap();
    let mut opd_iter = opds.values.chunks(n_mask).nth(0).unwrap().iter();
    dbg!(opds.values.len());
    let opd: Vec<_> = opds
        .mask
        .iter()
        .zip(&merged_mask)
        .filter_map(|(m, mm)| m.then(|| opd_iter.next().map(|o| (mm, o))).flatten())
        .filter_map(|(m, o)| m.then_some(o))
        .collect();
    println!("OPD: {}", opd.len());
    println!("OPD STD: {:.0}nm", std(opd.as_slice()) * 1e9);

    for n_kl in (100..=1000).step_by(100) {
        let c: Vec<f64> = kl_on_mask_orthonorm
            .chunks(n_merged_mask)
            .take(n_kl)
            .map(|kl| kl.iter().zip(&opd).map(|(a, &b)| a * b).sum::<f64>())
            .collect();
        // println!("c: {:?}", c);
        let opd_res = kl_on_mask_orthonorm.chunks(n_merged_mask).zip(&c).fold(
            opd.iter().map(|&x| *x).collect::<Vec<f64>>(),
            |mut a, (k, &c)| {
                a.iter_mut().zip(k).for_each(|(a, k)| *a -= k * c);
                a
            },
        );

        println!(
            "OPD RES STD: ({:4}) {:4.0}nm",
            n_kl,
            std(opd_res.iter().collect::<Vec<_>>().as_slice()) * 1e9
        );
    }

    Ok(())
}
