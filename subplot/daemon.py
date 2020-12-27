import logging
import os
import signal
import socket
import subprocess
import time


# Start a daemon that will open a port on localhost.
def daemon_start_on_port(ctx, path=None, args=None, name=None, port=None):
    daemon_start(ctx, path=path, args=args, name=name)
    daemon_wait_for_port("localhost", port)


# Start a daeamon, get its PID. Don't wait for a port or anything. This is
# meant for background processes that don't have port. Useful for testing the
# lib/daemon library of Subplot, but not much else.
def daemon_start(ctx, path=None, args=None, name=None):
    runcmd_run = globals()["runcmd_run"]
    runcmd_exit_code_is = globals()["runcmd_exit_code_is"]
    runcmd_get_exit_code = globals()["runcmd_get_exit_code"]
    runcmd_get_stderr = globals()["runcmd_get_stderr"]
    runcmd_prepend_to_path = globals()["runcmd_prepend_to_path"]

    argv = [path] + args.split()

    logging.debug(f"Starting daemon {name}")
    logging.debug(f"  ctx={ctx.as_dict()}")
    logging.debug(f"  name={name}")
    logging.debug(f"  path={path}")
    logging.debug(f"  args={args}")
    logging.debug(f"  argv={argv}")

    ns = ctx.declare("_daemon")

    this = ns[name] = {
        "pid-file": f"{name}.pid",
        "stderr": f"{name}.stderr",
        "stdout": f"{name}.stdout",
    }

    # Debian installs `daemonize` to /usr/sbin, which isn't part of the minimal
    # environment that Subplot sets up. So we add /usr/sbin to the PATH.
    runcmd_prepend_to_path(ctx, "/usr/sbin")
    runcmd_run(
        ctx,
        [
            "daemonize",
            "-c",
            os.getcwd(),
            "-p",
            this["pid-file"],
            "-e",
            this["stderr"],
            "-o",
            this["stdout"],
        ]
        + argv,
    )

    # Check that daemonize has exited OK. If it hasn't, it didn't start the
    # background process at all. If so, log the stderr in case there was
    # something useful there for debugging.
    exit = runcmd_get_exit_code(ctx)
    if exit != 0:
        stderr = runcmd_get_stderr(ctx)
        logging.error(f"daemon {name} stderr: {stderr}")
    runcmd_exit_code_is(ctx, 0)

    # Get the pid of the background process, from the pid file created by
    # daemonize. We don't need to wait for it, since we know daemonize already
    # exited. If it isn't there now, it's won't appear later.
    if not os.path.exists(this["pid-file"]):
        raise Exception("daemonize didn't create a PID file")

    this["pid"] = _daemon_wait_for_pid(this["pid-file"], 10.0)

    logging.debug(f"Started daemon {name}")
    logging.debug(f"  pid={this['pid']}")
    logging.debug(f"  ctx={ctx.as_dict()}")


def _daemon_wait_for_pid(filename, timeout):
    start = time.time()
    while time.time() < start + timeout:
        with open(filename) as f:
            data = f.read().strip()
            if data:
                return int(data)
    raise Exception("daemonize created a PID file without a PID")


def daemon_wait_for_port(host, port, timeout=3.0):
    addr = (host, port)
    try:
        s = socket.create_connection(addr, timeout=timeout)
    except socket.timeout:
        logging.error(f"daemon did not respond at port {port} within {timeout} seconds")
        raise
    except socket.error as e:
        logging.error(f"could not connect to daemon at {port}: {e}")
        raise
    s.close()


# Stop a daemon.
def daemon_stop(ctx, name=None):
    logging.debug(f"Stopping daemon {name}")
    ns = ctx.declare("_daemon")
    logging.debug(f"  ns={ns}")
    pid = ns[name]["pid"]
    signo = signal.SIGKILL

    logging.debug(f"Terminating process {pid} with signal {signo}")
    try:
        os.kill(pid, signo)
    except ProcessLookupError:
        logging.warning("Process did not actually exist (anymore?)")


def daemon_no_such_process(ctx, args=None):
    assert not _daemon_pgrep(args)


def daemon_process_exists(ctx, args=None):
    assert _daemon_pgrep(args)


def _daemon_pgrep(pattern):
    logging.info(f"checking if process exists: pattern={pattern}")
    exit = subprocess.call(["pgrep", "-laf", pattern])
    logging.info(f"exit code: {exit}")
    return exit == 0
