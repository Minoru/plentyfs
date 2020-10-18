#############################################################################
# Start and stop daemons, or background processes.


import logging
import os
import signal


# Start a process in the background.
def start_daemon(ctx, name, argv):
    runcmd_run = globals()["runcmd_run"]
    runcmd_exit_code_is = globals()["runcmd_exit_code_is"]

    logging.debug(f"Starting daemon {name}")
    logging.debug(f"  ctx={ctx.as_dict()}")
    logging.debug(f"  name={name}")
    logging.debug(f"  argv={argv}")

    if "daemon" not in ctx.as_dict():
        ctx["daemon"] = {}
    assert name not in ctx["daemon"]
    this = ctx["daemon"][name] = {
        "pid-file": f"{name}.pid",
        "stderr": f"{name}.stderr",
        "stdout": f"{name}.stdout",
    }
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
    runcmd_exit_code_is(ctx, 0)
    this["pid"] = int(open(this["pid-file"]).read().strip())
    assert process_exists(this["pid"])

    logging.debug(f"Started daemon {name}")
    logging.debug(f"  ctx={ctx.as_dict()}")


# Stop a daemon.
def stop_daemon(ctx, name):
    logging.debug(f"Stopping daemon {name}")
    logging.debug(f"  ctx={ctx.as_dict()}")
    logging.debug(f"  ctx['daemon']={ctx.as_dict()['daemon']}")

    this = ctx["daemon"][name]
    terminate_process(this["pid"], signal.SIGKILL)


# Does a process exist?
def process_exists(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


# Terminate process.
def terminate_process(pid, signalno):
    logging.debug(f"Terminating process {pid} with signal {signalno}")
    try:
        os.kill(pid, signalno)
    except ProcessLookupError:
        logging.debug("Process did not actually exist (anymore?)")
        pass
