use std::env;

fn main() {
    if option_env!("FEM_REPO").is_some() {
        println!("cargo:rustc-cfg=fem");
        let (input_names, _output_names) = gmt_fem_code_builder::io_names(env!("CARGO_PKG_NAME"))
            .expect("failed to get FEM inputs/ouputs names");
        if input_names.find("OSSGIRTooth6F").is_some() {
            println!("cargo:warning=OSSGIRTooth6F input detected in gmt_dos-client_mount crate");
            println!("cargo:rustc-cfg=gir_tooth")
        }
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
