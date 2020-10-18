# PlentyFS

A proof-of-concept for a read-only filesystem with random contents generated on
demand.

Developed [at the request of Lars Wirzenius](https://toot.liw.fi/@liw/105028618188354731).

Licensed under the [Blue Oak Model License 1.0.0](LICENSE.md).

## Dependencies

- Rust and Cargo (tested with Rust 1.47.0)
- fuse (including the headers, that's `libfuse-dev` on Debian) (tested with
    2.9.9)

## How to run

In the root of the repo:

```
mkdir mnt
cargo run --release mnt
```

This will mount PlentyFS into `mnt` directory and block. You can work with the
directory in a different terminal.

Once you're done, cd to the root of the repo and call:

```
fusermount -u mnt
```

This will unmount PlentyFS and release the first terminal.

## Architecture

### Requirements

The filesystem should:

- be read-only
- have files with random contents
- store (almost) nothing in memory or on disk

### The architecture under test

For this PoC, the FS contains a single directory (root) with a fixed number of
files (10,000) of fixed size (1 megabyte each). The only thing that's random is
the contents of the file.

Upon mounting, we take the PID of the fuse program. That's our "root seed", and
it's the only value that PlentyFS stores in memory. Everything else is computed
from it and the meta-information.

Upon each `read()`, we generate the blocks that contain requested data. To do
that, we:

1. generate a "file seed": combine root seed with the file's inode number, and
   hash the result;

2. generate the block: combine the file seed with the block number, and hash the
   result.

This architecture is "embarrassingly parallel", because the contents of the file
blocks depend only on the "root seed", file's inode, and the block offset. This
should enable it to scale linearly to many cores.

Details of this implementation:

- file inodes are numbered from 2 upwards;
- the hash we use is SHA-1.

### Benchmarks

On Intel i7-7700HQ, `tar -cvf /dev/shm/plentyfs.tar /mnt` achieves 115 MB/s.
Note that `/dev/shm/` is a `tempfs`. The speed was limited by this program,
which maxed out a single core for the whole duration of the benchmark.

## Why the name

Horn of plenty (conrnucopia) is a small object overflowing with food and riches.
Similarly, PlentyFS is a tiny filesystem containing as much data as you can
consume.
