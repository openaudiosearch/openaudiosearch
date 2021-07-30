use std::process::Command;
const FRONTEND_PATH: &str = "../../frontend";
fn main() {
    let frontend_path = if let Ok(path) = std::env::var("FRONTEND_PATH") {
        path
    } else {
        FRONTEND_PATH.to_string()
    };
    let frontend_dist_path = format!("{}/{}", frontend_path, "dist");
    println!("cargo:rerun-if-changed={}/src", frontend_path);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed={}", "BUILD_FRONTEND");
    println!("cargo:rerun-if-env-changed={}", "FRONTEND_PATH");
    println!("cargo:rerun-if-env-changed={}", "PROFILE");
    println!("frontend path: {}", frontend_path);
    let build_frontend = match std::env::var("BUILD_FRONTEND") {
        Ok(var) => match var.as_str() {
            "" | "0" | "false" => false,
            "1" | "true" => true,
            _ => panic!("Invalid value for BUILD_FRONTEND variable: should be 1 or 0"),
        },
        Err(_) => match std::env::var("PROFILE").unwrap().as_str() {
            "release" => true,
            "debug" => false,
            _ => false,
        },
    };
    if build_frontend {
        // In release mode, build the frontend using yarn. The result will be included statically
        // in the binary.
        Command::new("yarn")
            .current_dir(&frontend_path)
            .status()
            .expect("Failed to install frontend dependencies");
        Command::new("yarn")
            .current_dir(&frontend_path)
            .arg("build")
            .status()
            .expect("Failed to build frontend");
    } else {
        // On debug builds, don't rebuild the frontend but ensure that the directory is present.
        // Otherwise the build will fail because in src/server/mod.rs the frontend/dist directory
        // is statically included.
        std::fs::create_dir_all(&frontend_dist_path).expect("Failed to create frontend dist dir.");
    }
}
