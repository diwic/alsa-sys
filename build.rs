extern crate pkg_config;

fn main() {
    let missing: String = String::from("\"pkg-config\" \"--libs\" \"--cflags\" \"alsa\"");

    match pkg_config::probe_library("alsa") {
        Ok(_) => {},
        Err(e) => {
            match e {
                pkg_config::Error::Failure {command, output} => {
                    if command == missing {
                        panic! (
                            "Could not find the alsa library. Have you installed the library on your system?\n\n\
                            For Fedora users:\n# dnf install alsa-lib-devel\n\n\
                            For Debian/Ubuntu users:\n# apt-get install libasound2\n\n"
                        );
                    } else {
                        panic!("Command '{}' failed. Details:\n{:?}", command, output);
                    }
                },
                _ => panic!("{}", e)
            }
        }
    };
}
