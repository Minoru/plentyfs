use std::env;
use std::ffi::OsStr;

use getopts::Options;

use plentyfs::fs::PlentyFS;
use plentyfs::mountoptions::{MountOptions, UpdateError};

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn main() {
    let argv = env::args_os().collect::<Vec<_>>();

    let mut options = Options::new();
    options.optmulti("o", "", "mount options", "OPTIONS");
    let matches = match options.parse(&argv[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.free.is_empty() {
        eprintln!("Error: no mountpoint specified.");
        std::process::exit(EXIT_FAILURE);
    }
    let mountpoint = &matches.free[0];

    let mut mount_opts = MountOptions::default();
    for parameters in matches.opt_strs("o") {
        if let Err(error) = mount_opts.update_from(&parameters) {
            match error {
                UpdateError::NonHexValue { parameter, value } =>
                    eprintln!("Error: value `{}' for parameter `{}' is not a hexadecimal number.", value, parameter),

                UpdateError::NoValue { parameter } =>
                    eprintln!("Error: parameter `{}' requires a value.", parameter),

                UpdateError::ValueTooLong { parameter, value, max_allowed_length } =>
                    eprintln!("Error: value `{}' is too long for parameter `{}'; maximum allowed length is {} characters.", value, parameter, max_allowed_length),

                UpdateError::UnsupportedParameter { parameter, .. } =>
                    eprintln!("Error: parameter `{}' is not supported.", parameter),
            }
            std::process::exit(EXIT_FAILURE);
        }
    }

    let fuse_options = ["-o", "ro", "-o", "fsname=plentyfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuser::mount(PlentyFS::new(mount_opts.seed), &mountpoint, &fuse_options).unwrap();

    std::process::exit(EXIT_SUCCESS);
}
