use std::env;
use std::ffi::OsStr;

use plentyfs::PlentyFS;

fn main() {
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=plentyfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    // TODO: replace PID by a proper source of entropy. Add an option for the user to set their own
    // seed for reproducibility.
    fuser::mount(
        PlentyFS::new(std::process::id() as u64),
        &mountpoint,
        &options,
    )
    .unwrap();
}
