use std::process::Command;
const FRONTEND_PATH: &str = "../../frontend";
const FRONTEND_DIST_PATH: &str = "../../frontend/dist";
fn main() {
    println!("cargo:rerun-if-changed={}/src", FRONTEND_PATH);
    println!("cargo:rerun-if-env-changed={}", "BUILD_FRONTEND");
    println!("cargo:rerun-if-env-changed={}", "PROFILE");
    let build_frontend = match std::env::var("BUILD_FRONTEND") {
        Ok(_) => true,
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
            .current_dir(FRONTEND_PATH)
            .arg("build")
            .status()
            .expect("Failed to build frontend");
    } else {
        // On debug builds, don't rebuild the frontend but ensure that the directory is present.
        // Otherwise the build will fail because in src/server/mod.rs the frontend/dist directory
        // is statically included.
        std::fs::create_dir_all(FRONTEND_DIST_PATH).expect("Failed to create frontend dist dir.");
    }
}
