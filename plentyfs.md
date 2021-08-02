# Introduction

FIXME: explain what PlentyFS is at a very high level, and what the
motivation is

# Requirements

FIXME: explain the high level requirements as bullet points, not as
scenarios

# Acceptance criteria and their verification

This chapter covers the detailed acceptance criteria for PlentyFS and
how they are verified in an automated manner.

## Smoke test

This scenario verifies that PlentyFS works at all. We expect the root of
PlentyFS to contain 10,000 ordinary files and a directory called _.plentyfs_.

~~~scenario
given a PlentyFS mounted at mnt
then there are 10001 files under mnt
~~~


## If no mount point is specified, PlentyFS exits with an error

~~~scenario
when user runs PlentyFS without arguments
then exit code is 1
then stdout is empty
then stderr is exactly "Error: no mountpoint specified.\n"
~~~


## If mount option is not supported, PlentyFS exits with an error

~~~scenario
when user runs PlentyFS with arguments -o tunable=disabled mnt
then exit code is 1
then stdout is empty
then stderr is exactly "Error: parameter `tunable' is not supported.\n"
~~~


## User can specify a 64-bit seed

PlentyFS generates data at random, but it's a program, so it can't conjure true
randomness out of ether. Instead, it asks the operating system for a random
number, and generates the data from that. This initial random number is called
a "seed".

Some users might not want for the seed to be random. They might want to get the
same PlentyFS contents as they got some previous time. This could be useful when
reproducing bugs, or when writing tests — tasks that require a reproducible
environment.

To enable that, PlentyFS lets the user to optionally pass a seed as a mount
option.

### PlentyFS accepts `-o seed=<hex>` argument

~~~scenario
given a PlentyFS mounted at mnt with options seed=b1a914b7e0d996a8
then there are a file at mnt/1 that starts with 0x7da14ac962299c89
then there are a file at mnt/2 that starts with 0x2f016e5b70122476
then there are a file at mnt/3 that starts with 0xd8ab69b279bf68e2
then there are a file at mnt/5 that starts with 0xae6efee6824678c2
~~~

### Errors if seed is empty

There are a couple different ways in which the seed could be empty:

1. the user could type just the name of the parameter:

    ~~~scenario
    when user runs PlentyFS with arguments -o seed mnt
    then exit code is 1
    then stdout is empty
    then stderr is exactly "Error: parameter `seed' requires a value.\n"
    ~~~

2. the user could type the name of the parameter and the `=` delimiter, but no
   actual value:

    ~~~scenario
    when user runs PlentyFS with arguments -o seed= mnt
    then exit code is 1
    then stdout is empty
    then stderr is exactly "Error: parameter `seed' requires a value.\n"
    ~~~

### Errors if seed is longer than 16 characters

~~~scenario
when user runs PlentyFS with arguments -o seed=abba0110f00dba7abba0202 mnt
then exit code is 1
then stdout is empty
then stderr is exactly "Error: value `abba0110f00dba7abba0202' is too long for parameter `seed'; maximum allowed length is 16 characters.\n"
~~~

### Errors if seed is not hexadecimal

~~~scenario
when user runs PlentyFS with arguments -o seed=random mnt
then exit code is 1
then stdout is empty
then stderr is exactly "Error: value `random' for parameter `seed' is not a hexadecimal number.\n"
~~~


## PlentyFS provides its current seed via _.plentyfs/seed_

As described in the previous section, the contents of a mounted PlentyFS
filesystem are derived from a single 64-bit number, called "seed". If the user
doesn't provide it, PlentyFS asks the operating system to generate one. But what
if the user finds the resulting filesystem so interesting that they want to
re-create it later? To facilitate that use-case, PlentyFS provides its current
seed via a file called _.plentyfs/seed_.

### _.plentyfs/seed_ is not empty even if user didn't provide a seed

~~~scenario
given a PlentyFS mounted at mnt
then file mnt/.plentyfs/seed is not empty
~~~

### _.plentyfs/seed_ contains the seed provided on the command line

~~~scenario
given a PlentyFS mounted at mnt with options seed=2490d7f7528f40b7
then file mnt/.plentyfs/seed contains "2490d7f7528f40b7"
~~~


---
title: PlentyFS - read-only, on-demand file system
author: Alexander Batischev
bindings:
- subplot/plentyfs.yaml
- lib/files.yaml
- lib/runcmd.yaml
template: python
functions:
- subplot/plentyfs.py
- lib/daemon.py
- lib/files.py
- lib/runcmd.py
...
