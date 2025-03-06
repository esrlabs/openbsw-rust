use std::env;
use std::path::PathBuf;

fn main() {
    let builder = bindgen::Builder::default()
        .use_core()
        .header("wrapper.h")
        .clang_arg("-I../include")
        .clang_arg("-I../../estd/include")
        .clang_arg("-I../../platform/include")
        // .clang_arg(format!("-I{}", env::var("CMAKE_INCLUDE_PATH").unwrap()))
        .clang_args(["-x", "c++"])
        .clang_arg("-std=c++14")
        .clang_arg("--stdlib=libstdc++");
    let builder = match env::var("TARGET").unwrap().as_str() {
        "thumbv7em-none-eabihf" => {
            // FIXME this needs to be generalized. Maybe there is a generic way to also support clang?
            builder
                .clang_arg("--sysroot=/nix/store/yr14jnxwq8jry78hc9fk21j88r4jc749-gcc-arm-embedded-12.3.rel1/arm-none-eabi")
                .clang_args(["-Wno-psabi", "-fno-asynchronous-unwind-tables", "-fno-builtin", "-fshort-enums", "-mcpu=cortex-m4", "-funsigned-bitfields", "-fmessage-length=0", "-fno-common", "-ffunction-sections", "-fdata-sections", "-mthumb", "-mfloat-abi=hard", "-mfpu=fpv4-sp-d16", "-mno-unaligned-access"])
                .clang_arg("-I/nix/store/yr14jnxwq8jry78hc9fk21j88r4jc749-gcc-arm-embedded-12.3.rel1/arm-none-eabi/include/c++/12.3.1/arm-none-eabi/thumb/v7e-m+fp/hard")
        }
        _ => builder,
    };
    let bindings = builder
        // TODO Only allow the wanted types
        // .enable_cxx_namespaces()
        // .allowlist_item("can_.*")
        // .allowlist_type("can_CANFrame")
        // .allowlist_item("::can::CANFrame")
        // .allowlist_type("root::can::CanId")
        .opaque_type("std::.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
