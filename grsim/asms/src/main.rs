use gmt_fem::FEM;

fn main() {
    let mut fem = FEM::from_env()?;
    println!("{fem}");
}
