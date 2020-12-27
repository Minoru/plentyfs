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

This scenario verifies that PlentyFS works at all.

~~~scenario
given a PlentyFS mounted at mnt
then there are 10000 files under mnt
~~~


## If no mount point is specified, PlentyFS exits with an error

~~~scenario
when user runs PlentyFS without arguments
then exit code is 1
then stdout is empty
then stderr is exactly "Error: no mountpoint specified.\n"
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

~~~scenario
given a PlentyFS mounted at mnt with options seed=b1a914b7e0d996a8
then there are a file at mnt/1 that starts with 0x1df2c952085c9471
then there are a file at mnt/2 that starts with 0x7e56b30ef2e3d19b
then there are a file at mnt/3 that starts with 0xf2319fa6ef40322c
then there are a file at mnt/5 that starts with 0x307de35f4d9ad2a1
~~~


---
title: PlentyFS - read-only, on-demand file system
author: Alexander Batischev
template: python
bindings:
- subplot/plentyfs.yaml
- subplot/runcmd.yaml
functions:
- subplot/daemon.py
- subplot/plentyfs.py
- subplot/runcmd.py
...

