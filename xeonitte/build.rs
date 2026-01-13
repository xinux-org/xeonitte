use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-env-changed=RESULT_PATH");

    let result_path = env::var("RESULT_PATH")
        .expect("RESULT_PATH environment variable not set");

    let path = Path::new("src/config.rs");

    let original = fs::read_to_string(path)
        .expect("Failed to read src/config.rs");

    let new_content = original.replace("/* RESULT_PATH */", &format!("\"{}\", ", result_path));

    fs::write(path, new_content)
        .expect("Failed to write src/config.rs");
}
