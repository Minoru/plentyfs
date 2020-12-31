import logging
import os
import re
import time


def files_create_from_embedded(ctx, filename=None):
    files_create_from_embedded_with_other_name(
        ctx, filename_on_disk=filename, embedded_filename=filename
    )


def files_create_from_embedded_with_other_name(
    ctx, filename_on_disk=None, embedded_filename=None
):
    get_file = globals()["get_file"]
    with open(filename_on_disk, "wb") as f:
        f.write(get_file(embedded_filename))


def files_create_from_text(ctx, filename=None, text=None):
    with open(filename, "w") as f:
        f.write(text)


def files_file_exists(ctx, filename=None):
    assert_eq = globals()["assert_eq"]
    assert_eq(os.path.exists(filename), True)


def files_file_does_not_exist(ctx, filename=None):
    assert_eq = globals()["assert_eq"]
    assert_eq(os.path.exists(filename), False)


def files_only_these_exist(ctx, filenames=None):
    assert_eq = globals()["assert_eq"]
    filenames = filenames.replace(",", "").split()
    assert_eq(set(os.listdir(".")), set(filenames))


def files_file_contains(ctx, filename=None, data=None):
    assert_eq = globals()["assert_eq"]
    with open(filename, "rb") as f:
        actual = f.read()
        actual = actual.decode("UTF-8")
    assert_eq(data in actual, True)


def files_file_matches_regex(ctx, filename=None, regex=None):
    assert_eq = globals()["assert_eq"]
    with open(filename) as f:
        content = f.read()
    m = re.search(regex, content)
    if m is None:
        logging.debug(f"files_file_matches_regex: no match")
        logging.debug(f"  filenamed: {filename}")
        logging.debug(f"  regex: {regex}")
        logging.debug(f"  content: {regex}")
        logging.debug(f"  match: {m}")
    assert_eq(bool(m), True)


def files_match(ctx, filename1=None, filename2=None):
    assert_eq = globals()["assert_eq"]
    with open(filename1, "rb") as f:
        data1 = f.read()
    with open(filename2, "rb") as f:
        data2 = f.read()
    assert_eq(data1, data2)


def files_touch_with_timestamp(
    ctx,
    filename=None,
    year=None,
    month=None,
    day=None,
    hour=None,
    minute=None,
    second=None,
):
    t = (
        int(year),
        int(month),
        int(day),
        int(hour),
        int(minute),
        int(second),
        -1,
        -1,
        -1,
    )
    ts = time.mktime(t)
    _files_touch(filename, ts)


def files_touch(ctx, filename=None):
    _files_touch(filename, None)


def _files_touch(filename, ts):
    if not os.path.exists(filename):
        open(filename, "w").close()
    times = None
    if ts is not None:
        times = (ts, ts)
    os.utime(filename, times=times)


def files_mtime_is_recent(ctx, filename=None):
    st = os.stat(filename)
    age = abs(st.st_mtime - time.time())
    assert age < 1.0


def files_mtime_is_ancient(ctx, filename=None):
    st = os.stat(filename)
    age = abs(st.st_mtime - time.time())
    year = 365 * 24 * 60 * 60
    required = 39 * year
    logging.debug(f"ancient? mtime={st.st_mtime} age={age} required={required}")
    assert age > required


def files_remember_metadata(ctx, filename=None):
    meta = _files_remembered(ctx)
    meta[filename] = _files_get_metadata(filename)
    logging.debug("files_remember_metadata:")
    logging.debug(f"  meta: {meta}")
    logging.debug(f"  ctx: {ctx}")


# Check that current metadata of a file is as stored in the context.
def files_has_remembered_metadata(ctx, filename=None):
    assert_eq = globals()["assert_eq"]
    meta = _files_remembered(ctx)
    logging.debug("files_has_remembered_metadata:")
    logging.debug(f"  meta: {meta}")
    logging.debug(f"  ctx: {ctx}")
    assert_eq(meta[filename], _files_get_metadata(filename))


def files_has_different_metadata(ctx, filename=None):
    assert_ne = globals()["assert_ne"]
    meta = _files_remembered(ctx)
    assert_ne(meta[filename], _files_get_metadata(filename))


def _files_remembered(ctx):
    ns = ctx.declare("_files")
    return ns.get("remembered-metadata", {})


def _files_get_metadata(filename):
    st = os.lstat(filename)
    keys = ["st_dev", "st_gid", "st_ino", "st_mode", "st_mtime", "st_size", "st_uid"]
    return {key: getattr(st, key) for key in keys}
