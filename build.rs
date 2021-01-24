extern crate pkg_config;

#[cfg(feature = "use-bindgen")]
extern crate bindgen;

#[allow(unused_variables)]
fn main() {
    let alsa_library = match pkg_config::Config::new().statik(false).probe("alsa") {
        Err(e) => {
            match e {
                pkg_config::Error::Failure { .. } => panic! (
                    "Pkg-config failed - usually this is because alsa development headers are not installed.\n\n\
                    For Fedora users:\n# dnf install alsa-lib-devel\n\n\
                    For Debian/Ubuntu users:\n# apt-get install libasound2-dev\n\n\
                    pkg_config details:\n{}",
                    e
                ),
                _ => panic!("{}", e)
            }
        },
        Ok(l) => l
    };

    #[cfg(feature = "use-bindgen")]
    generate_bindings(&alsa_library);
}

#[cfg(feature = "use-bindgen")]
fn generate_bindings(alsa_library: &pkg_config::Library) {
    use std::env;
    use std::path::PathBuf;

    let clang_include_args = alsa_library.include_paths.iter().map(|include_path| {
        format!(
            "-I{}",
            include_path.to_str().expect("include path was not UTF-8")
        )
    });

    let mut codegen_config = bindgen::CodegenConfig::empty();
    codegen_config.insert(bindgen::CodegenConfig::FUNCTIONS);
    codegen_config.insert(bindgen::CodegenConfig::TYPES);

    let builder = bindgen::Builder::default()
        .size_t_is_usize(true)
        .whitelist_recursively(false)
        .prepend_enum_name(false)
        .layout_tests(false)
        .whitelist_function("snd_.*")
        .whitelist_type("_?snd_.*")
        .with_codegen_config(codegen_config)
        .clang_args(clang_include_args)
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("generated.rs"))
        .expect("Couldn't write bindings");
}
