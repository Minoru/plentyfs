import logging
import os
import time


def mount_plentyfs(ctx, dirname=None):
    logging.info(f"starting plentyfs at {dirname}")

    srcdir = globals()["srcdir"]
    daemon_start = globals()["daemon_start"]

    os.mkdir(dirname)
    plentyfs = os.path.join(srcdir, "target", "debug", "plentyfs")
    daemon_start(ctx, plentyfs, dirname)

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
