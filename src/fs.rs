use std::convert::TryInto;
use std::ffi::OsStr;

use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use sha1::Sha1;
use std::time::{Duration, UNIX_EPOCH};

const FILES_COUNT: u64 = 10_000;
const FILE_SIZE: u64 = 1_048_576; // bytes (1 megabyte)
const BLOCK_SIZE: usize = 20; // bytes (the size of SHA-1 digest)

// This is the default on modern drives, and we don't care much about this value anyway because we
// haven't done any performance tuning yet.
const REPORTED_BLOCK_SIZE: u32 = 4096;

const TTL: Duration = Duration::from_secs(1);

/// Attributes of the root directory of the filesystem.
const ROOT_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o555,
    nlink: 2,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
    blksize: REPORTED_BLOCK_SIZE,
    padding: 0,
};

/// Attributes of the _.plentyfs_ directory.
const DOT_PLENTYFS_DIR_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o555,
    nlink: 2,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
    blksize: REPORTED_BLOCK_SIZE,
    padding: 0,
};

/// Attributes of a file inside _.plentyfs_ directory.
const CONFIG_FILE_ATTR: FileAttr = FileAttr {
    ino: 3,
    size: FILE_SIZE,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o444,
    nlink: 1,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
    blksize: REPORTED_BLOCK_SIZE,
    padding: 0,
};

/// Attributes of an ordinary data file in the filesystem.
const DATA_FILE_ATTR: FileAttr = FileAttr {
    ino: 101,
    size: FILE_SIZE,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o444,
    nlink: 1,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
    blksize: REPORTED_BLOCK_SIZE,
    padding: 0,
};

/// A structure representing a mounted instance of PlentyFS.
pub struct PlentyFS {
    /// Initial value of our bespoke RNG.
    seed: u64,
}

impl PlentyFS {
    /// Create new instance with a given seed.
    pub fn new(seed: u64) -> PlentyFS {
        PlentyFS { seed }
    }
}

impl Filesystem for PlentyFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent != 1  && parent != 2 {
            reply.error(ENOENT);
            return;
        }

        match filename_to_inode(parent, name).and_then(|ino| inode_to_file_attr(ino)) {
            Some(attr) => reply.entry(&TTL, &attr, 0),
            None => reply.error(ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match inode_to_file_attr(ino) {
            Some(attr) => reply.attr(&TTL, &attr),
            None => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        if ino >= 3 && ino <= 100 {
            if ino == 3 {
                // This is .plentyfs/seed file.
                let seed = format!("{:08x}\n", self.seed);
                let bytes = seed.as_bytes();

                let start = (offset as usize).min(bytes.len());
                let end = ((offset + size as i64) as usize).min(bytes.len());
                reply.data(&bytes.get(start .. end).unwrap_or(&[]));
            } else {
                reply.error(ENOENT);
            }
        } else if ino >= 101 && ino <= (FILES_COUNT + 100) {
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
        match ino {
            1 => {
                if offset == 0 {
                    // TODO: handle possibility of buffer being full
                    let _ = reply.add(1, 1, FileType::Directory, ".");
                }

                if offset <= 1 {
                    // TODO: handle possibility of buffer being full
                    let _ = reply.add(1, 1, FileType::Directory, "..");
                }

                if offset <= 2 {
                    // TODO: handle possibility of buffer being full
                    let _ = reply.add(2, 1, FileType::RegularFile, ".plentyfs");
                }

                if offset >= 3 {
                    offset -= 3;
                }

                for file_number in (0..FILES_COUNT).skip(offset as usize) {
                    let inode = file_number + 101;
                    let next_entry_offset = file_number + 4;
                    // TODO: handle possibility of buffer being full
                    let _ = reply.add(
                        inode,
                        next_entry_offset as i64,
                        FileType::RegularFile,
                        file_number.to_string(),
                    );
                }
                reply.ok();
            }

            2 => {
                // This is .plentyfs directory.

                if offset == 0 {
                    // TODO: handle possibility of buffer being full
                    let _ = reply.add(2, 1, FileType::Directory, ".");
                }

                if offset <= 1 {
                    // TODO: handle possibility of buffer being full
                    let _ = reply.add(1, 1, FileType::Directory, "..");
                }

                if offset <= 2 {
                    // TODO: handle possibility of buffer being full
                    let _ = reply.add(3, 3, FileType::RegularFile, "seed");
                }

                reply.ok();
            }

            _ => reply.error(ENOENT),
        }
    }
}

/// Convert filename into an inode number. Note that the inode might not actually exist.
///
/// Returns `None` if conversion couldn't be performed (most probably because the filename is
/// malformed).
fn filename_to_inode(parent_inode: u64, name: &OsStr) -> Option<u64> {
    let name = name.to_str()?;

    match parent_inode {
        1 => {
            // root directory
            match name {
                ".plentyfs" => Some(2),
        s => s
            .parse::<u64>()
            .ok()
            // Filenames start at 0, but inodes for these files start at 101.
            .and_then(|file_number| Some(file_number + 101)),
            }
        },
        2 => {
            // .plentyfs
            match name {
                "seed" => Some(3),
                _ => None,
            }
        },
        _ => None,
    }
}

/// Convert inode number into a `FileAttr` structure.
///
/// Returns `None` if the file doesn't exist.
fn inode_to_file_attr(ino: u64) -> Option<FileAttr> {
    if ino == 1 {
        Some(ROOT_DIR_ATTR)
    } else if ino == 2 {
        Some(DOT_PLENTYFS_DIR_ATTR)
    } else if ino >= 3 && ino <= 100 {
        Some(FileAttr {
            ino,
            ..CONFIG_FILE_ATTR
        })
    } else if ino >= 101 && ino <= (FILES_COUNT + 100) {
        Some(FileAttr {
            ino,
            ..DATA_FILE_ATTR
        })
    } else {
        None
    }
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
