fn main () {
    if cfg!(feature = "use-bindgen") {
        use std::fs::copy;
        use std::path::PathBuf;
        let bindings_path = concat!(env!("OUT_DIR"), "/generated.rs");
        copy(PathBuf::from(bindings_path), PathBuf::from("src/generated.rs")).unwrap();
    } else {
        panic!("Must be run with use-bindgen feature");
    }
}
