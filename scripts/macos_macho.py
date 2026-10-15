#!/usr/bin/env python3
"""Inspect and rewrite macOS Mach-O load commands for the bundled RustFS binary.

RustFS 1.0.1's Apple silicon asset records one non-system dependency:

    LC_LOAD_DYLIB /opt/homebrew/opt/xz/lib/liblzma.5.dylib
        (compatibility version 14.0.0, current version 14.3.0)

dyld then aborts with "Library missing" on a Mac that does not have Homebrew's
xz. DiagnosticReports often print that path as /opt/homebrew/*/liblzma.5.dylib.
The asterisk is the crash reporter redacting the middle components; it is not
the install name stored in the file.

A load path is relocatable when it is under /usr/lib or /System, or when it
starts with @executable_path/, @loader_path/, or @rpath/. liblzma.5.dylib is
the one dependency the launcher vendors beside the binary. Every other
non-relocatable path fails the build.
"""

from __future__ import annotations

import json
import struct
import sys
from dataclasses import dataclass

MH_MAGIC_64 = 0xFEEDFACF
FAT_MAGIC = 0xCAFEBABE
FAT_CIGAM = 0xBEBAFECA

LC_LOAD_DYLIB = 0xC
LC_ID_DYLIB = 0xD
LC_PREBOUND_DYLIB = 0x10
LC_LOAD_WEAK_DYLIB = 0x80000018
LC_RPATH = 0x8000001C
LC_REEXPORT_DYLIB = 0x8000001F
LC_LAZY_LOAD_DYLIB = 0x20
LC_LOAD_UPWARD_DYLIB = 0x80000023

LIBLZMA_FILE = "liblzma.5.dylib"
LIBLZMA_LOAD = "@loader_path/liblzma.5.dylib"

DEPENDENCY_COMMANDS = {
    LC_LOAD_DYLIB: "LC_LOAD_DYLIB",
    LC_LOAD_WEAK_DYLIB: "LC_LOAD_WEAK_DYLIB",
    LC_REEXPORT_DYLIB: "LC_REEXPORT_DYLIB",
    LC_LOAD_UPWARD_DYLIB: "LC_LOAD_UPWARD_DYLIB",
    LC_LAZY_LOAD_DYLIB: "LC_LAZY_LOAD_DYLIB",
    LC_PREBOUND_DYLIB: "LC_PREBOUND_DYLIB",
}
VERSIONED_COMMANDS = {
    LC_LOAD_DYLIB,
    LC_ID_DYLIB,
    LC_LOAD_WEAK_DYLIB,
    LC_REEXPORT_DYLIB,
    LC_LOAD_UPWARD_DYLIB,
    LC_LAZY_LOAD_DYLIB,
}
COMMAND_NAMES = {
    **DEPENDENCY_COMMANDS,
    LC_ID_DYLIB: "LC_ID_DYLIB",
    LC_RPATH: "LC_RPATH",
}


@dataclass(frozen=True)
class Load:
    command: str
    cmd: int
    path: str
    compatibility: int
    current: int
    command_at: int
    name_at: int
    name_capacity: int

    def version_text(self) -> str:
        return (
            f"{self.command} {self.path} "
            f"(compatibility version {format_version(self.compatibility)}, "
            f"current version {format_version(self.current)})"
        )


def format_version(packed: int) -> str:
    return f"{packed >> 16}.{(packed >> 8) & 0xFF}.{packed & 0xFF}"


def parse_version(text: str) -> int:
    major, minor, patch = (int(part) for part in text.split("."))
    return (major << 16) | (minor << 8) | patch


def is_relocatable(path: str) -> bool:
    return path.startswith(
        ("@executable_path/", "@loader_path/", "@rpath/", "/usr/lib/", "/System/")
    )


def file_name(path: str) -> str:
    return path.rsplit("/", 1)[-1]


def _u32(data: bytes, offset: int, little: bool) -> int:
    endian = "<I" if little else ">I"
    if offset < 0 or offset + 4 > len(data):
        raise ValueError(f"Mach-O truncated at {offset}")
    return struct.unpack_from(endian, data, offset)[0]


def _slices(data: bytes) -> list[tuple[int, int]]:
    if len(data) < 8:
        raise ValueError("not a Mach-O file")
    magic_le = _u32(data, 0, True)
    magic_be = _u32(data, 0, False)
    if magic_be == FAT_MAGIC:
        count = _u32(data, 4, False)
        slices = []
        cursor = 8
        for _ in range(count):
            if cursor + 20 > len(data):
                raise ValueError("fat header truncated")
            _cputype, _subtype, offset, size, _align = struct.unpack_from(
                ">IIIII", data, cursor
            )
            slices.append((offset, size))
            cursor += 20
        return slices
    if magic_be == FAT_CIGAM:
        raise ValueError("little-endian fat Mach-O is not supported")
    if magic_le == MH_MAGIC_64:
        return [(0, len(data))]
    if magic_le in (0xFEEDFACE, 0xCEFAEDFE, 0xCFFAEDFE):
        raise ValueError(
            "unsupported Mach-O encoding; refusing to bundle it without reading its load commands"
        )
    raise ValueError("not a Mach-O file")


def is_macho(data: bytes) -> bool:
    if len(data) < 4:
        return False
    magic_le = _u32(data, 0, True)
    magic_be = _u32(data, 0, False)
    return magic_le == MH_MAGIC_64 or magic_be in (FAT_MAGIC, FAT_CIGAM) or magic_le in (
        0xFEEDFACE,
        0xCEFAEDFE,
        0xCFFAEDFE,
    )


def _loads_in_slice(data: bytes, base: int, size: int) -> list[Load]:
    if base < 0 or size < 32 or base + 32 > len(data):
        raise ValueError("Mach-O slice truncated")
    if _u32(data, base, True) != MH_MAGIC_64:
        raise ValueError("Mach-O slice is not 64-bit little-endian")
    ncmds = _u32(data, base + 16, True)
    sizeofcmds = _u32(data, base + 20, True)
    if sizeofcmds > size - 32 or base + 32 + sizeofcmds > len(data):
        raise ValueError("Mach-O load commands extend past the slice")
    loads: list[Load] = []
    cursor = base + 32
    end = cursor + sizeofcmds
    for _ in range(ncmds):
        if cursor + 8 > end:
            raise ValueError("Mach-O load command truncated")
        cmd = _u32(data, cursor, True)
        cmdsize = _u32(data, cursor + 4, True)
        if cmdsize < 8 or cursor + cmdsize > end:
            raise ValueError(f"Mach-O load command size {cmdsize} is invalid")
        if cmd in COMMAND_NAMES:
            if cursor + 12 > end:
                raise ValueError("Mach-O dylib command truncated")
            name_off = _u32(data, cursor + 8, True)
            if name_off < 8 or name_off >= cmdsize:
                raise ValueError("Mach-O dylib name offset is outside the command")
            name_at = cursor + name_off
            raw = data[name_at : cursor + cmdsize]
            if b"\x00" not in raw:
                raise ValueError("Mach-O dylib path is not NUL-terminated")
            path = raw.split(b"\x00", 1)[0].decode("utf-8")
            current = compatibility = 0
            if cmd in VERSIONED_COMMANDS:
                current = _u32(data, cursor + 16, True)
                compatibility = _u32(data, cursor + 20, True)
            loads.append(
                Load(
                    command=COMMAND_NAMES[cmd],
                    cmd=cmd,
                    path=path,
                    compatibility=compatibility,
                    current=current,
                    command_at=cursor,
                    name_at=name_at,
                    name_capacity=cmdsize - name_off,
                )
            )
        cursor += cmdsize
    return loads


def load_commands(data: bytes) -> list[Load]:
    loads: list[Load] = []
    for offset, size in _slices(data):
        loads.extend(_loads_in_slice(data, offset, size))
    return loads


def dependencies(loads: list[Load]) -> list[Load]:
    return [item for item in loads if item.cmd in DEPENDENCY_COMMANDS or item.cmd == LC_RPATH]


def liblzma_loads(loads: list[Load]) -> list[Load]:
    return [
        item
        for item in loads
        if item.cmd in DEPENDENCY_COMMANDS and file_name(item.path) == LIBLZMA_FILE
    ]


def unresolved_loads(loads: list[Load]) -> list[Load]:
    """Load commands that still cannot be opened on a Mac without Homebrew."""
    unresolved = []
    for item in dependencies(loads):
        if item.cmd != LC_RPATH and file_name(item.path) == LIBLZMA_FILE:
            if item.path != LIBLZMA_LOAD:
                unresolved.append(item)
            continue
        if not is_relocatable(item.path):
            unresolved.append(item)
    return unresolved


def format_blocked(loads: list[Load]) -> str:
    lines = [
        "macOS RustFS binary is not relocatable.",
        "These load commands point outside /usr/lib, /System, and @executable_path/@loader_path/@rpath:",
    ]
    lines.extend(f"  {item.version_text()}" for item in loads)
    lines.append(
        'dyld aborts at launch with "Library missing" unless that exact file exists. '
        "DiagnosticReports may print a middle path component as '*'; the command above is the path in the file."
    )
    return "\n".join(lines)


def rewrite_path(data: bytearray, old: str, new: str, *, ident: bool = False) -> int:
    changed = 0
    encoded = new.encode("utf-8")
    for item in load_commands(bytes(data)):
        if ident:
            if item.cmd != LC_ID_DYLIB or item.path == new:
                continue
        elif item.path != old:
            continue
        if len(encoded) + 1 > item.name_capacity:
            raise ValueError(
                f"cannot replace {item.path} with {new}: the load command has room for {item.name_capacity - 1} bytes"
            )
        data[item.name_at : item.name_at + item.name_capacity] = b"\x00" * item.name_capacity
        data[item.name_at : item.name_at + len(encoded)] = encoded
        changed += 1
    if changed == 0 and not ident:
        raise ValueError(f"load command not found: {old}")
    return changed


def _dump(item: Load) -> dict[str, str]:
    return {
        "command": item.command,
        "path": item.path,
        "compatibility": format_version(item.compatibility),
        "current": format_version(item.current),
    }


def plan(data: bytes) -> dict[str, object]:
    if not is_macho(data):
        return {"status": "not-macho", "liblzma": [], "blocked": []}
    try:
        loads = load_commands(data)
    except ValueError as error:
        return {"status": "error", "message": str(error), "liblzma": [], "blocked": []}
    liblzma = liblzma_loads(loads)
    unresolved = unresolved_loads(loads)
    if not unresolved:
        status = "clean"
    elif all(
        item.cmd != LC_RPATH and file_name(item.path) == LIBLZMA_FILE for item in unresolved
    ):
        status = "vendor-lzma"
    else:
        status = "blocked"
    ident = next((item for item in loads if item.cmd == LC_ID_DYLIB), None)
    return {
        "status": status,
        "message": format_blocked(unresolved) if unresolved else "",
        "liblzma": [_dump(item) for item in liblzma],
        "blocked": [_dump(item) for item in unresolved],
        "id": _dump(ident) if ident is not None else None,
        "required_compatibility": format_version(
            max((item.compatibility for item in liblzma), default=0)
        ),
    }


def accept_liblzma(dylib: bytes, required_compat: int) -> str | None:
    """Return an error string when the vendored library cannot satisfy the binary."""
    try:
        loads = load_commands(dylib)
    except ValueError as error:
        return f"vendored {LIBLZMA_FILE} is not a readable Mach-O: {error}"
    ident = next((item for item in loads if item.cmd == LC_ID_DYLIB), None)
    if ident is None:
        return f"vendored {LIBLZMA_FILE} has no LC_ID_DYLIB"
    if ident.path != LIBLZMA_LOAD:
        return (
            f"vendored {LIBLZMA_FILE} install name is {ident.path}, expected {LIBLZMA_LOAD}"
        )
    if ident.compatibility < required_compat:
        return (
            f"vendored {LIBLZMA_FILE} compatibility version {format_version(ident.compatibility)} "
            f"is older than the {format_version(required_compat)} required by RustFS"
        )
    bad = [
        item
        for item in dependencies(loads)
        if not is_relocatable(item.path)
    ]
    if bad:
        return "vendored liblzma is itself not relocatable:\n" + format_blocked(bad)
    return None


def build_fixture(paths: list[str], *, ident: str | None = None, compat: int = 14 << 16, current: int = (14 << 16) | (3 << 8)) -> bytes:
    commands: list[tuple[int, str, bool]] = []
    if ident is not None:
        commands.append((LC_ID_DYLIB, ident, True))
    for path in paths:
        commands.append((LC_LOAD_DYLIB, path, True))
    encoded_commands = bytearray()
    for cmd, path, versioned in commands:
        raw = path.encode("utf-8") + b"\x00"
        cmdsize = 24 + len(raw) if versioned else 12 + len(raw)
        cmdsize = (cmdsize + 7) & ~7
        blob = bytearray(cmdsize)
        struct.pack_into("<III", blob, 0, cmd, cmdsize, 24 if versioned else 12)
        if versioned:
            struct.pack_into("<III", blob, 12, 2, current, compat)
            blob[24 : 24 + len(raw)] = raw
        else:
            blob[12 : 12 + len(raw)] = raw
        encoded_commands.extend(blob)
    header = struct.pack(
        "<IIIIIIII",
        MH_MAGIC_64,
        0x0100000C,
        0,
        2,
        len(commands),
        len(encoded_commands),
        0x200085,
        0,
    )
    return header + bytes(encoded_commands)


def _read(path: str) -> bytes:
    with open(path, "rb") as handle:
        return handle.read()


def _self_test() -> None:
    system = build_fixture(["/usr/lib/libSystem.B.dylib", "/System/Library/Frameworks/Foundation.framework/Foundation"])
    assert plan(system)["status"] == "clean", plan(system)

    homebrew = "/opt/homebrew/opt/xz/lib/liblzma.5.dylib"
    original = bytearray(build_fixture([homebrew, "/usr/lib/libSystem.B.dylib"]))
    planned = plan(bytes(original))
    assert planned["status"] == "vendor-lzma", planned
    assert planned["liblzma"][0]["path"] == homebrew
    assert planned["required_compatibility"] == "14.0.0"
    rewrite_path(original, homebrew, LIBLZMA_LOAD)
    assert plan(bytes(original))["status"] == "clean"
    assert liblzma_loads(load_commands(bytes(original)))[0].path == LIBLZMA_LOAD

    starred = bytearray(build_fixture(["/opt/homebrew/*/liblzma.5.dylib"]))
    assert plan(bytes(starred))["status"] == "vendor-lzma"
    rewrite_path(starred, "/opt/homebrew/*/liblzma.5.dylib", LIBLZMA_LOAD)
    assert plan(bytes(starred))["status"] == "clean"

    mixed = build_fixture(
        [homebrew, "/usr/local/opt/openssl@3/lib/libssl.3.dylib"]
    )
    mixed_plan = plan(mixed)
    assert mixed_plan["status"] == "blocked", mixed_plan
    message = str(mixed_plan["message"])
    assert homebrew in message
    assert "libssl.3.dylib" in message
    starred_other = plan(build_fixture(["/opt/homebrew/*/libfoo.dylib"]))
    assert starred_other["status"] == "blocked"
    assert "/opt/homebrew/*/libfoo.dylib" in str(starred_other["message"])

    dylib = bytearray(build_fixture(["/usr/lib/libSystem.B.dylib"], ident="/usr/local/lib/liblzma.5.dylib"))
    rewrite_path(dylib, "", LIBLZMA_LOAD, ident=True)
    error = accept_liblzma(bytes(dylib), 14 << 16)
    assert error is None, error
    old = bytearray(build_fixture(["/usr/lib/libSystem.B.dylib"], ident=LIBLZMA_LOAD, compat=6 << 16))
    assert accept_liblzma(bytes(old), 14 << 16)

    tiny = build_fixture(["/x"])
    try:
        rewrite_path(bytearray(tiny), "/x", LIBLZMA_LOAD)
    except ValueError as error:
        assert "room for" in str(error)
    else:
        raise AssertionError("short load command should not grow")

    try:
        load_commands(b"\xcf\xfa\xed\xfe" + b"\x00" * 4)
    except ValueError:
        pass
    else:
        raise AssertionError("truncated macho should fail")

    print("macos_macho self-test passed")


def main(argv: list[str]) -> int:
    if len(argv) >= 2 and argv[1] == "--self-test":
        _self_test()
        return 0
    if len(argv) < 3:
        print(
            "usage: macos_macho.py plan|check|needs-lzma|is-macho|rewrite|rewrite-id|accept-lzma|write-fixture ...",
            file=sys.stderr,
        )
        return 2
    command, path = argv[1], argv[2]
    if command == "write-fixture":
        payload = build_fixture(argv[3:] or ["/usr/lib/libSystem.B.dylib"])
        with open(path, "wb") as handle:
            handle.write(payload)
        return 0

    data = _read(path)
    if command == "is-macho":
        return 0 if is_macho(data) else 1
    if command == "plan":
        print(json.dumps(plan(data), separators=(",", ":")))
        return 0
    if command == "check":
        info = plan(data)
        if info["status"] in ("blocked", "vendor-lzma", "error"):
            print(info.get("message") or info["status"], file=sys.stderr)
            return 1
        return 0
    if command == "needs-lzma":
        info = plan(data)
        if info["status"] == "error":
            print(info["message"], file=sys.stderr)
            return 2
        print("yes" if info["liblzma"] else "no")
        return 0
    if command == "rewrite":
        if len(argv) != 5:
            print("usage: macos_macho.py rewrite FILE OLD NEW", file=sys.stderr)
            return 2
        blob = bytearray(data)
        try:
            count = rewrite_path(blob, argv[3], argv[4])
        except ValueError as error:
            print(error, file=sys.stderr)
            return 1
        with open(path, "wb") as handle:
            handle.write(blob)
        print(f"rewrote {count} load command(s): {argv[3]} -> {argv[4]}")
        return 0
    if command == "rewrite-id":
        if len(argv) != 4:
            print("usage: macos_macho.py rewrite-id FILE NEW", file=sys.stderr)
            return 2
        blob = bytearray(data)
        try:
            count = rewrite_path(blob, "", argv[3], ident=True)
        except ValueError as error:
            print(error, file=sys.stderr)
            return 1
        if count == 0:
            print(f"LC_ID_DYLIB already {argv[3]}")
            return 0
        with open(path, "wb") as handle:
            handle.write(blob)
        print(f"rewrote LC_ID_DYLIB -> {argv[3]}")
        return 0
    if command == "accept-lzma":
        if len(argv) != 4:
            print("usage: macos_macho.py accept-lzma DYLIB RUSTFS", file=sys.stderr)
            return 2
        rustfs = plan(_read(argv[3]))
        if rustfs["status"] == "error":
            print(rustfs["message"], file=sys.stderr)
            return 1
        required = parse_version(str(rustfs["required_compatibility"]))
        error = accept_liblzma(data, required)
        if error:
            print(error, file=sys.stderr)
            return 1
        return 0
    print(f"unknown command {command}", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
