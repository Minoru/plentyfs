use std::env;
use std::ffi::OsStr;

use getopts::Options;

use plentyfs::PlentyFS;

fn main() {
    let argv = env::args_os().collect::<Vec<_>>();

    let options = Options::new();
    let matches = match options.parse(&argv[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.free.is_empty() {
        eprintln!("No mountpoint specified.");
        return;
    }
    let mountpoint = &matches.free[0];

    let fuse_options = ["-o", "ro", "-o", "fsname=plentyfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    // TODO: replace PID by a proper source of entropy. Add an option for the user to set their own
    // seed for reproducibility.
    fuser::mount(
        PlentyFS::new(std::process::id() as u64),
        &mountpoint,
        &fuse_options,
    )
    .unwrap();
}
