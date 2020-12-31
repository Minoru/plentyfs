import logging
import os
import re
import shlex
import subprocess


#
# Helper functions.
#

# Get exit code or other stored data about the latest command run by
# runcmd_run.


def _runcmd_get(ctx, name):
    ns = ctx.declare("_runcmd")
    return ns[name]


def runcmd_get_exit_code(ctx):
    return _runcmd_get(ctx, "exit")


def runcmd_get_stdout(ctx):
    return _runcmd_get(ctx, "stdout")


def runcmd_get_stdout_raw(ctx):
    return _runcmd_get(ctx, "stdout.raw")


def runcmd_get_stderr(ctx):
    return _runcmd_get(ctx, "stderr")


def runcmd_get_stderr_raw(ctx):
    return _runcmd_get(ctx, "stderr.raw")


def runcmd_get_argv(ctx):
    return _runcmd_get(ctx, "argv")


# Run a command, given an argv and other arguments for subprocess.Popen.
#
# This is meant to be a helper function, not bound directly to a step. The
# stdout, stderr, and exit code are stored in the "_runcmd" namespace in the
# ctx context.
def runcmd_run(ctx, argv, **kwargs):
    ns = ctx.declare("_runcmd")

    # The Subplot Python template empties os.environ at startup, modulo a small
    # number of variables with carefully chosen values. Here, we don't need to
    # care about what those variables are, but we do need to not overwrite
    # them, so we just add anything in the env keyword argument, if any, to
    # os.environ.
    env = dict(os.environ)
    for key, arg in kwargs.pop("env", {}).items():
        env[key] = arg

    pp = ns.get("path-prefix")
    if pp:
        env["PATH"] = pp + ":" + env["PATH"]

    logging.debug(f"runcmd_run")
    logging.debug(f"  argv: {argv}")
    logging.debug(f"  env: {env}")
    p = subprocess.Popen(
        argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, **kwargs
    )
    stdout, stderr = p.communicate("")
    ns["argv"] = argv
    ns["stdout.raw"] = stdout
    ns["stderr.raw"] = stderr
    ns["stdout"] = stdout.decode("utf-8")
    ns["stderr"] = stderr.decode("utf-8")
    ns["exit"] = p.returncode
    logging.debug(f"  ctx: {ctx}")
    logging.debug(f"  ns: {ns}")


# Step: prepend srcdir to PATH whenever runcmd runs a command.
def runcmd_helper_srcdir_path(ctx):
    srcdir = globals()["srcdir"]
    runcmd_prepend_to_path(ctx, srcdir)


# Step: This creates a helper script.
def runcmd_helper_script(ctx, filename=None):
    get_file = globals()["get_file"]
    with open(filename, "wb") as f:
        f.write(get_file(filename))


#
# Step functions for running commands.
#


def runcmd_prepend_to_path(ctx, dirname=None):
    ns = ctx.declare("_runcmd")
    pp = ns.get("path-prefix", "")
    if pp:
        pp = f"{pp}:{dirname}"
    else:
        pp = dirname
    ns["path-prefix"] = pp


def runcmd_step(ctx, argv0=None, args=None):
    runcmd_try_to_run(ctx, argv0=argv0, args=args)
    runcmd_exit_code_is_zero(ctx)


def runcmd_try_to_run(ctx, argv0=None, args=None):
    argv = [shlex.quote(argv0)]
    if args is not None:
        argv.extend(shlex.split(args))
    runcmd_run(ctx, argv)


#
# Step functions for examining exit codes.
#


def runcmd_exit_code_is_zero(ctx):
    runcmd_exit_code_is(ctx, exit=0)


def runcmd_exit_code_is(ctx, exit=None):
    assert_eq = globals()["assert_eq"]
    assert_eq(runcmd_get_exit_code(ctx), int(exit))


def runcmd_exit_code_is_nonzero(ctx):
    runcmd_exit_code_is_not(ctx, exit=0)


def runcmd_exit_code_is_not(ctx, exit=None):
    assert_ne = globals()["assert_ne"]
    assert_ne(runcmd_get_exit_code(ctx), int(exit))


#
# Step functions and helpers for examining output in various ways.
#


def runcmd_stdout_is(ctx, text=None):
    _runcmd_output_is(runcmd_get_stdout(ctx), text)


def runcmd_stdout_isnt(ctx, text=None):
    _runcmd_output_isnt(runcmd_get_stdout(ctx), text)


def runcmd_stderr_is(ctx, text=None):
    _runcmd_output_is(runcmd_get_stderr(ctx), text)


def runcmd_stderr_isnt(ctx, text=None):
    _runcmd_output_isnt(runcmd_get_stderr(ctx), text)


def _runcmd_output_is(actual, wanted):
    assert_eq = globals()["assert_eq"]
    wanted = bytes(wanted, "utf8").decode("unicode_escape")
    logging.debug("_runcmd_output_is:")
    logging.debug(f"  actual: {actual!r}")
    logging.debug(f"  wanted: {wanted!r}")
    assert_eq(actual, wanted)


def _runcmd_output_isnt(actual, wanted):
    assert_ne = globals()["assert_ne"]
    wanted = bytes(wanted, "utf8").decode("unicode_escape")
    logging.debug("_runcmd_output_isnt:")
    logging.debug(f"  actual: {actual!r}")
    logging.debug(f"  wanted: {wanted!r}")
    assert_ne(actual, wanted)


def runcmd_stdout_contains(ctx, text=None):
    _runcmd_output_contains(runcmd_get_stdout(ctx), text)


def runcmd_stdout_doesnt_contain(ctx, text=None):
    _runcmd_output_doesnt_contain(runcmd_get_stdout(ctx), text)


def runcmd_stderr_contains(ctx, text=None):
    _runcmd_output_contains(runcmd_get_stderr(ctx), text)


def runcmd_stderr_doesnt_contain(ctx, text=None):
    _runcmd_output_doesnt_contain(runcmd_get_stderr(ctx), text)


def _runcmd_output_contains(actual, wanted):
    assert_eq = globals()["assert_eq"]
    wanted = bytes(wanted, "utf8").decode("unicode_escape")
    logging.debug("_runcmd_output_contains:")
    logging.debug(f"  actual: {actual!r}")
    logging.debug(f"  wanted: {wanted!r}")
    assert_eq(wanted in actual, True)


def _runcmd_output_doesnt_contain(actual, wanted):
    assert_ne = globals()["assert_ne"]
    wanted = bytes(wanted, "utf8").decode("unicode_escape")
    logging.debug("_runcmd_output_doesnt_contain:")
    logging.debug(f"  actual: {actual!r}")
    logging.debug(f"  wanted: {wanted!r}")
    assert_ne(wanted in actual, True)


def runcmd_stdout_matches_regex(ctx, regex=None):
    _runcmd_output_matches_regex(runcmd_get_stdout(ctx), regex)


def runcmd_stdout_doesnt_match_regex(ctx, regex=None):
    _runcmd_output_doesnt_match_regex(runcmd_get_stdout(ctx), regex)


def runcmd_stderr_matches_regex(ctx, regex=None):
    _runcmd_output_matches_regex(runcmd_get_stderr(ctx), regex)


def runcmd_stderr_doesnt_match_regex(ctx, regex=None):
    _runcmd_output_doesnt_match_regex(runcmd_get_stderr(ctx), regex)


def _runcmd_output_matches_regex(actual, regex):
    assert_ne = globals()["assert_ne"]
    r = re.compile(regex)
    m = r.search(actual)
    logging.debug("_runcmd_output_matches_regex:")
    logging.debug(f"  actual: {actual!r}")
    logging.debug(f"  regex: {regex!r}")
    logging.debug(f"  match: {m}")
    assert_ne(m, None)


def _runcmd_output_doesnt_match_regex(actual, regex):
    assert_eq = globals()["assert_eq"]
    r = re.compile(regex)
    m = r.search(actual)
    logging.debug("_runcmd_output_doesnt_match_regex:")
    logging.debug(f"  actual: {actual!r}")
    logging.debug(f"  regex: {regex!r}")
    logging.debug(f"  match: {m}")
    assert_eq(m, None)
