extern crate pkg_config;

#[cfg(feature = "use-bindgen")]
extern crate bindgen;

fn main() {
    match pkg_config::Config::new().statik(false).probe("alsa") {
        Err(pkg_config::Error::Failure { command, output }) => panic!(
            "Pkg-config failed - usually this is because alsa development headers are not installed.\n\n\
            For Fedora users:\n# dnf install alsa-lib-devel\n\n\
            For Debian/Ubuntu users:\n# apt-get install libasound2-dev\n\n\
            For Archlinux users:\n# pacman -S alsa-lib\n\n
            pkg_config details:\n{}\n", pkg_config::Error::Failure { command, output }),
        Err(e) => panic!("{}", e),
        Ok(alsa_library) => {
            probe_time64(&alsa_library);
            #[cfg(feature = "use-bindgen")]
            generate_bindings(&alsa_library);
        }
    };
}

// 32-bit glibc's snd_htimestamp_t (a struct timespec) is 8 or 16 bytes
// depending on the time64 transition; libc::timespec assumes 8, so ALSA can
// write past it and segfault on 32-bit targets with a 64-bit time_t.
fn probe_time64(alsa_library: &pkg_config::Library) {
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
    let include_paths = &alsa_library.include_paths;

    // Measures the exact type libasound writes through. If even this control
    // snippet does not compile, there is no safe width to assume: guessing 8
    // can segfault and guessing 16 yields garbage timestamps, so stop the
    // build with a diagnosable error instead.
    if let Err(e) = try_compile(
        &out_dir,
        include_paths,
        "time64_control.c",
        "#include <alsa/asoundlib.h>\n\
         _Static_assert(sizeof(snd_htimestamp_t) == 8\n\
                     || sizeof(snd_htimestamp_t) == 16, \"unexpected snd_htimestamp_t\");\n",
    ) {
        panic!(
            "alsa-sys could not compile a probe measuring sizeof(snd_htimestamp_t) \
             against the ALSA headers. On 32-bit glibc targets that size decides \
             whether libasound writes 8 or 16 bytes per timestamp, and guessing \
             wrong corrupts memory, so the build stops instead.\n\
             Check that the C compiler and the ALSA headers for the target work.\n\n\
             {}",
            e
        );
    }

    // 16 bytes exactly when the 64-bit time_t ABI is in effect, from the port
    // default or from _TIME_BITS=64 in the toolchain, as on Debian trixie.
    // __TIMESIZE sees neither: it stays 32 on every architecture that went
    // through the transition. Compiles but never runs, so cross builds work.
    if try_compile(
        &out_dir,
        include_paths,
        "time64_probe.c",
        "#include <alsa/asoundlib.h>\n\
         _Static_assert(sizeof(snd_htimestamp_t) == 16, \"64-bit time_t\");\n",
    )
    .is_ok()
    {
        println!("cargo:rustc-cfg=alsa_sys_time64");
    }
}

/// Compiles a snippet against the target's ALSA headers. Nothing is linked
/// or run.
fn try_compile(
    out_dir: &std::path::Path,
    include_paths: &[std::path::PathBuf],
    name: &str,
    source: &str,
) -> Result<(), String> {
    let path = out_dir.join(name);
    std::fs::write(&path, source).map_err(|e| e.to_string())?;
    cc::Build::new()
        .file(&path)
        .includes(include_paths)
        .cargo_metadata(false)
        .cargo_warnings(false)
        .warnings(false)
        .try_compile(name.trim_end_matches(".c"))
        .map_err(|e| e.to_string())
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
