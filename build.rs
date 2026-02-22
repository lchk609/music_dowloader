use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let mut libs = HashMap::new();

    libs.insert("material".into(), PathBuf::from("material"));

    let config: slint_build::CompilerConfiguration =
        slint_build::CompilerConfiguration::new().with_library_paths(libs);

    slint_build::compile_with_config("ui/app.slint", config).expect("Slint build failed");
}
