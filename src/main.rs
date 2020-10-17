use std::convert::TryInto;
use std::env;
use std::ffi::OsStr;

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use sha1::Sha1;
use time::Timespec;

const FILES_COUNT: u64 = 10_000;
const FILE_SIZE: u64 = 1_048_576; // bytes (1 megabyte)
const BLOCK_SIZE: usize = 20; // bytes (the size of SHA-1 digest)

const TTL: Timespec = Timespec { sec: 1, nsec: 0 };

const ROOT_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
};

const UNIX_EPOCH: Timespec = Timespec { sec: 0, nsec: 0 };

const FILE_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: FILE_SIZE,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
};

struct MazeFS {
    /// Initial value of our bespoke RNG.
    seed: u64,
}

impl Filesystem for MazeFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent != 1 {
            reply.error(ENOENT);
            return;
        }

        match name.to_str().and_then(|s| s.parse::<u64>().ok()) {
            Some(file_number) => {
                if file_number < FILES_COUNT {
                    reply.entry(&TTL, &FILE_ATTR, 0);
                } else {
                    reply.error(ENOENT);
                }
            }

            None => reply.error(ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if ino == 1 {
            reply.attr(&TTL, &ROOT_DIR_ATTR);
        } else if ino >= 2 && ino <= (FILES_COUNT + 1) {
            reply.attr(&TTL, &FILE_ATTR);
        } else {
            reply.error(ENOENT);
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        if ino >= 2 && ino <= (FILES_COUNT + 1) {
            let seed = generate_file_seed(self.seed, ino);

            let first_block = (offset as u64) / (BLOCK_SIZE as u64);
            let blocks_count = (size as u64 + 2 * BLOCK_SIZE as u64) / (BLOCK_SIZE as u64);

            let mut data = vec![];
            for block_no in first_block..(first_block + blocks_count) {
                data.extend(&generate_block_data(seed, block_no));
            }

            let inside_offset = (offset as usize) % (BLOCK_SIZE as usize);
            reply.data(&data[inside_offset..(inside_offset + size as usize)]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        mut offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        if offset == 0 {
            reply.add(1, 1, FileType::Directory, ".");
        }

        if offset <= 1 {
            reply.add(1, 1, FileType::Directory, ".");
        }

        if offset >= 2 {
            offset -= 2;
        }

        for file_number in (0..FILES_COUNT).skip(offset as usize) {
            let inode = file_number + 2;
            let next_entry_offset = 2 + file_number + 1;
            reply.add(
                inode,
                next_entry_offset as i64,
                FileType::RegularFile,
                file_number.to_string(),
            );
        }
        reply.ok();
    }
}

fn main() {
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=mazefs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    // TODO: replace PID by a proper source of entropy. Add an option for the user to set their own
    // seed for reproducibility.
    fuse::mount(
        MazeFS {
            seed: std::process::id() as u64,
        },
        &mountpoint,
        &options,
    )
    .unwrap();
}

fn generate_file_seed(root_seed: u64, inode: u64) -> u64 {
    let file_salt = "filesalt".as_bytes();
    let (file_salt, _) = file_salt.split_at(std::mem::size_of::<u64>());
    // TODO: refactor to make panics impossible.
    let file_salt: [u8; 8] = file_salt.try_into().unwrap();
    let file_salt = u64::from_le_bytes(file_salt);

    // TODO: replace XOR with a better mixing technique.
    root_seed ^ inode ^ file_salt
}

fn generate_block_data(seed: u64, block_no: u64) -> [u8; BLOCK_SIZE] {
    let mut sha1 = Sha1::from(seed.to_le_bytes());
    sha1.update(&block_no.to_le_bytes());
    sha1.digest().bytes()
}
