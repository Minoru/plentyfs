use std::env;
use std::ffi::OsStr;

use fuse::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use time::Timespec;

const FILES_COUNT: u64 = 1_000;

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
    size: 0,
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

struct MazeFS;

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
        _offset: i64,
        _size: u32,
        reply: ReplyData,
    ) {
        if ino >= 2 && ino <= (FILES_COUNT + 1) {
            reply.data(&[]);
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
    fuse::mount(MazeFS, &mountpoint, &options).unwrap();
}
