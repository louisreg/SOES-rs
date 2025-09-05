#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

fn main() {
    let include_path = "src/soes-c/soes-esi";

    // Compile C-code
    cc::Build::new()
        .file("./src/soes-c/esc.c")
        .file("./src/soes-c/esc_foe.c")
        .file("./src/soes-c/esc_eoe.c")
        .file("./src/soes-c/esc_eep.c")
        .file("./src/soes-c/esc_coe.c")
        .file("./src/soes-c/ecat_slv.c")
        .file("./src/soes-c/objectlist.c")
        .include("./src/soes-c")
        .flag_if_supported("-Wno-address-of-packed-member")
        .compile("soes");

    // Generate binders
    let bindings = bindgen::Builder::default()
        .header("./src/soes-c/esc.h")
        .header("./src/soes-c/esc_foe.h")
        .header("./src/soes-c/esc_eoe.h")
        .header("./src/soes-c/esc_eep.h")
        .header("./src/soes-c/esc_coe.h")
        .header("./src/soes-c/ecat_slv.h")
        .clang_arg("-Isrc/soes-c/soes-esi/")
        .clang_arg("-Isrc/soes-c/")
        .use_core()
        .ctypes_prefix("cty")
        .layout_tests(false)
        .generate_comments(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
