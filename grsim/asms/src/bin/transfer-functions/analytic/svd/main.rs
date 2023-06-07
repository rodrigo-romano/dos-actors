use asms::Sys;
use std::{env, fs::File, io::BufWriter, path::Path, time::Instant};

fn main() -> anyhow::Result<()> {
    let root = env::args()
        .skip(1)
        .next()
        .expect("expected 1 argument, found none");

    let repo = env::var("DATA_REPO").unwrap();
    let path = Path::new(&repo).join(format!("{}.tgz", root));

    println!("Loading system from {:?}", path);
    let now = Instant::now();
    let sys = Sys::from_tarball(&path)?;
    println!("System loaded in {}s", now.elapsed().as_secs());

    println!("Computing the system singular values");
    let now = Instant::now();
    let sys_sv = sys.singular_values();
    println!("Singular values computed in {}s", now.elapsed().as_secs());

    let nu = sys.frequencies();

    let path = Path::new(&repo).join(format!("{}_sv.pkl", root));
    let file = File::create(path)?;
    let mut buffer = BufWriter::new(file);
    serde_pickle::to_writer(&mut buffer, &(nu, sys_sv), Default::default())?;

    Ok(())
}
