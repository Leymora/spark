use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target = "--target=".to_owned() + &out_dir + "/resources/resources.gresource";
    let mkdir_target = out_dir.to_owned() + "/resources";

    let _mkdir_status = Command::new("mkdir")
    .args(&[&mkdir_target])
    .status().expect("Failed to make resource directory");

    let compile_status = Command::new("glib-compile-resources")
    .args(&["--sourcedir=src/resources", &target, "src/resources/resources.gresource.xml"])
    .status().expect("glib-compile failed to compile resources");

    println!("cargo::rustc-link-search=native={}", out_dir);
    println!("cargo::rerun-if-changed=build.rs");
}
