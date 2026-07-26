extern crate pkg_config;

#[cfg(feature = "use-bindgen")]
extern crate bindgen;

fn main() {
    probe_time64();

    match pkg_config::Config::new().statik(false).probe("alsa") {
        Err(pkg_config::Error::Failure { command, output }) => panic!(
            "Pkg-config failed - usually this is because alsa development headers are not installed.\n\n\
            For Fedora users:\n# dnf install alsa-lib-devel\n\n\
            For Debian/Ubuntu users:\n# apt-get install libasound2-dev\n\n\
            For Archlinux users:\n# pacman -S alsa-lib\n\n
            pkg_config details:\n{}\n", pkg_config::Error::Failure { command, output }),
        Err(e) => panic!("{}", e),
        Ok(_alsa_library) => {
            #[cfg(feature = "use-bindgen")]
            generate_bindings(&_alsa_library);
        }
    };
}

// 32-bit glibc's struct timespec is 8 or 16 bytes depending on the time64
// transition; libc::timespec assumes 8, so ALSA can write past it and
// segfault on 32-bit targets with time64.
fn probe_time64() {
    println!("cargo:rustc-check-cfg=cfg(alsa_sys_time64)");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV");
    let target_pointer_width = std::env::var("CARGO_CFG_TARGET_POINTER_WIDTH");

    // Only glibc on 32-bit Linux has this ambiguity. musl defaulted to a
    // 64-bit time_t from the start, and 64-bit targets already match
    // libc::timespec natively.
    if target_os.is_ok_and(|v| v != "linux")
        || target_env.is_ok_and(|v| v != "gnu")
        || target_pointer_width.is_ok_and(|v| v != "32")
    {
        return;
    }

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Checks the type, not just that a compiler exists: an unusable <time.h>
    // would fail the probe below and be read as pre-transition, which is the
    // direction that segfaults.
    if !compiles(
        &out_dir,
        "time64_control.c",
        "#include <time.h>\n\
         _Static_assert(sizeof(struct timespec) == 8\n\
                     || sizeof(struct timespec) == 16, \"unexpected timespec\");\n",
    ) {
        println!("cargo:warning=alsa-sys: could not probe glibc's time_t width, assuming legacy 32-bit time_t");
        return;
    }

    // 16 bytes exactly when the 64-bit time_t ABI is in effect, from the port
    // default or from _TIME_BITS=64 in the toolchain, as on Debian trixie.
    // __TIMESIZE sees neither: it stays 32 on every architecture that went
    // through the transition. Compiles but never runs, so cross builds work.
    if compiles(
        &out_dir,
        "time64_probe.c",
        "#include <time.h>\n\
         _Static_assert(sizeof(struct timespec) == 16, \"64-bit time_t\");\n",
    ) {
        println!("cargo:rustc-cfg=alsa_sys_time64");
    }
}

/// Compiles a snippet against the target's headers. Nothing is linked or run.
fn compiles(out_dir: &std::path::Path, name: &str, source: &str) -> bool {
    let path = out_dir.join(name);
    if std::fs::write(&path, source).is_err() {
        return false;
    }
    cc::Build::new()
        .file(&path)
        .cargo_metadata(false)
        .cargo_warnings(false)
        .warnings(false)
        .try_compile(name.trim_end_matches(".c"))
        .is_ok()
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
    codegen_config.insert(bindgen::CodegenConfig::VARS);

    let builder = bindgen::Builder::default()
        .use_core()
        .size_t_is_usize(true)
        .allowlist_recursively(false)
        .prepend_enum_name(false)
        .layout_tests(false)
        .allowlist_function("snd_.*")
        .allowlist_type("_?snd_.*")
        .allowlist_type(".*va_list.*")
        .allowlist_var("SND_.*")
        .fit_macro_constants(false)
        .with_codegen_config(codegen_config)
        .clang_args(clang_include_args)
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("generated.rs"))
        .expect("Couldn't write bindings");
}
