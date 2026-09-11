use std::env;
use std::path::PathBuf;

fn main() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .flag_if_supported("-Wno-unused-parameter")
        .file("rubberband/single/RubberBandSingle.cpp");
    println!("cargo:rerun-if-changed=rubberband/single/RubberBandSingle.cpp");

    // default on Apple platforms uses vDSP in the Accelerate framework
    if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    if cfg!(target_os = "windows") {
        build
            .define("_WIN32", None)
            .define("NOMINMAX", None)
            .define("_USE_MATH_DEFINES", None)
            .define("GETOPT_API", "");
    } else {
        build
            .define("USE_PTHREADS", None)
            .define("HAVE_POSIX_MEMALIGN", None);
    }

    build.compile("rubberband");

    let bindings = bindgen::Builder::default()
        .header("rubberband/rubberband/rubberband-c.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate rubberband bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write rubberband bindings");
}
