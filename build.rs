#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

fn main() {
    // Compile C-code
    cc::Build::new()
        .file("./src/soes-c/esc.c")
        .file("./src/soes-c/esc_foe.c")
        .file("./src/soes-c/esc_eoe.c")
        .file("./src/soes-c/esc_eep.c")
        .file("./src/soes-c/esc_coe.c")
        .file("./src/soes-c/tinyprintf.c")
        .file("./src/soes-c/objectlist.c")
        .include("./src/soes-c")
        .define("EC_LITTLE_ENDIAN", None)
        .flag_if_supported("-Wno-address-of-packed-member")
        .compile("soes");

    // Generate binders
    let bindings = bindgen::Builder::default()
        .header("./src/soes-c/esc.h")
        .clang_arg("-Isrc/soes-c") // include path for headers
        .clang_arg("-Isrc/soes-c/soes-esi")
        .clang_arg("--target=arm-none-eabi")
        .clang_arg("-DEC_LITTLE_ENDIAN") // force little endian for conditional fields
        .clang_arg("-DCC_PACKED=") // ignore CC_PACKED macros
        .clang_arg("-DCC_PACKED_BEGIN=")
        .clang_arg("-DCC_PACKED_END=")
        .clang_arg("-nostdinc") // optional, avoid using system includes
        .clang_arg("-I/usr/local/arm-none-eabi/bin/../arm-none-eabi/include/")
        .clang_arg("-I/usr/local/arm-none-eabi/lib/gcc/arm-none-eabi/14.2.1/include/")
        .clang_arg("--sysroot=/usr/local/arm-none-eabi/bin/../arm-none-eabi/")
        .use_core() // use core instead of std
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
