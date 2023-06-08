use std::env;

fn main() {
    if option_env!("FEM_REPO").is_some() {
        println!("cargo:rustc-cfg=fem");
    }
    if let Ok(_) = env::var("MOUNT_FDR_AZ17HZ") {
        println!("cargo:warning=compiling ODC mount control with 17Hz notch filter");
    } else {
        println!("cargo:warning=compiling vanilla ODC mount control");
    };
    println!("cargo:rerun-if-env-changed=MOUNT_FDR_AZ17HZ");
    println!("cargo:rerun-if-env-changed=FEM_REPO");
    println!("cargo:rerun-if-changed=build.rs");
}
