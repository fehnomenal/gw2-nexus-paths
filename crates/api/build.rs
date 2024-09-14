use std::env;
use std::path::PathBuf;

fn main() {
    dbg!(env::var("BINDGEN_EXTRA_CLANG_ARGS").unwrap());

    let bindings = bindgen::Builder::default()
        .header("nexus-api/Nexus.h")
        .header("mumble-api/Mumble.h")
        .clang_arg("-xc++")
        .clang_arg(format!(
            "--target={}",
            env::var("CARGO_BUILD_TARGET").unwrap()
        ))
        .generate_cstr(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
