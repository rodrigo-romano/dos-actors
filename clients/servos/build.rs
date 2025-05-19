fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        println!("cargo::rustc-env=MOUNT_MODEL=MOUNT_FDR_1kHz");
    }
    gmt_fem_code_builder::rustc_config(env!("CARGO_PKG_NAME"), None).unwrap();
    println!("cargo:rerun-if-env-changed=FEM_REPO");
    println!("cargo:rerun-if-changed=build.rs");
}
