import logging
import os
import time


def mount_plentyfs(ctx, dirname=None, options=None):
    logging.info(f"starting plentyfs at {dirname} with options {options}")

    srcdir = globals()["srcdir"]
    daemon_start = globals()["daemon_start"]

    os.mkdir(dirname)
    plentyfs = os.path.join(srcdir, "target", "debug", "plentyfs")

    if options is None:
        argv = dirname
    else:
        argv = f"-o {options} -- {dirname}"

    daemon_start(ctx, plentyfs, argv)

    # Wait for plentyfs to have started, up to two seconds.
    started = time.time()
    timeout = 2.0
    while time.time() < started + timeout:
        if os.listdir(dirname):
            break

    ctx["mount-point"] = dirname


def unmount_plentyfs(ctx):
    runcmd_run = globals()["runcmd_run"]
    runcmd_exit_code_is = globals()["runcmd_exit_code_is"]

    dirname = ctx["mount-point"]
    logging.info(f"stopping plentyfs at {dirname}")
    runcmd_run(ctx, ["fusermount", "-u", dirname])
    runcmd_exit_code_is(ctx, 0)


def run_plentyfs(ctx, arguments=None):
    runcmd_try_to_run = globals()["runcmd_try_to_run"]
    srcdir = globals()["srcdir"]
    plentyfs = os.path.join(srcdir, "target", "debug", "plentyfs")
    runcmd_try_to_run(ctx, plentyfs, arguments)


def file_count_is(ctx, count=None, dirname=None):
    logging.debug(f"counting files under {dirname}")
    n = 0
    for path, subdirs, basenames in os.walk(dirname):
        logging.debug(f"path: {path}")
        logging.debug(f"subdirs: {subdirs}")
        logging.debug(f"basenames: {basenames}")
        at_path = len(basenames) + len(subdirs)
        logging.debug(f"under {path}: {at_path}")
        n += at_path

    assert_eq = globals()["assert_eq"]
    assert_eq(int(count), n)


def stdout_is_empty(ctx):
    runcmd_get_stdout = globals()["runcmd_get_stdout"]
    assert_eq = globals()["assert_eq"]

    stdout = runcmd_get_stdout(ctx)
    assert_eq(stdout, "")

def file_has_prefix(ctx, path=None, prefix=None):
    binary_prefix = bytes.fromhex(prefix)

    with open(path, "rb") as f:
        actual_prefix = f.read(len(binary_prefix))

    assert_eq = globals()["assert_eq"]
    assert_eq(binary_prefix, actual_prefix)
