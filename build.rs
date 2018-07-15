extern crate pkg_config;

fn main() {
    if let Err(e) = pkg_config::probe_library("alsa") {
        match e {
            pkg_config::Error::Failure { .. } => panic! (
                "Could not find the alsa library. Have you installed the library on your system?\n\n\
                For Fedora users:\n# dnf install alsa-lib-devel\n\n\
                For Debian/Ubuntu users:\n# apt-get install libasound2-dev\n\n\
                pkg_config details:\n{}",
                e
            ),
            _ => panic!("{}", e)
        }
    }
}
