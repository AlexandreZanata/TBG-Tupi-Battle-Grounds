"""Inspect ARM64 ELF imports and APK library alignment without executing game code."""

import struct
import zipfile

SYSTEM_LIBRARIES = {
    "libandroid.so", "liblog.so", "libdl.so", "libm.so", "libc.so",
    "libvulkan.so", "libEGL.so", "libGLESv2.so", "libGLESv3.so",
    "libOpenSLES.so", "libaaudio.so", "libz.so", "libjnigraphics.so",
}


def elf_identity(raw, abi="arm64-v8a"):
    """Require a bounded ELF64 ARM64 shared object and inspect all LOAD segments."""
    if len(raw) < 64 or raw[:6] != b"\x7fELF\x02\x01":
        raise ValueError("Expected a little-endian ELF64 shared library")
    kind, machine = struct.unpack_from("<HH", raw, 16)
    expected_machine={"arm64-v8a":183,"x86_64":62}.get(abi)
    if kind != 3 or machine != expected_machine:
        raise ValueError("Expected an ARM64 ET_DYN library")
    offset = struct.unpack_from("<Q", raw, 32)[0]
    size, count = struct.unpack_from("<HH", raw, 54)
    if size != 56 or not 1 <= count <= 128 or offset + count * size > len(raw):
        raise ValueError("Invalid ELF program-header table")
    headers = [struct.unpack_from("<IIQQQQQQ", raw, offset + i * size) for i in range(count)]
    loads = [h for h in headers if h[0] == 1]
    if not loads:
        raise ValueError("ELF has no LOAD segments")
    for _, _, file_offset, address, _, file_size, memory_size, alignment in loads:
        if (file_offset + file_size > len(raw) or memory_size < file_size
                or alignment < 16384 or alignment & (alignment - 1)
                or file_offset % alignment != address % alignment):
            raise ValueError("ELF LOAD is invalid or not aligned for 16 KB pages")
    dynamic = [h for h in headers if h[0] == 2]
    if len(dynamic) != 1:
        raise ValueError("Expected one ELF dynamic table")
    start, length = dynamic[0][2], dynamic[0][5]
    if start + length > len(raw) or length % 16:
        raise ValueError("Invalid ELF dynamic table")
    needed, string_address, string_size = [], None, None
    terminated = False
    for at in range(start, start + length, 16):
        tag, value = struct.unpack_from("<qQ", raw, at)
        if tag == 0:
            terminated = True
            break
        if tag == 1:
            needed.append(value)
        elif tag == 5:
            string_address = value
        elif tag == 10:
            string_size = value
    if not terminated or string_address is None or string_size is None:
        raise ValueError("ELF dynamic string table is missing")
    strings = None
    for h in loads:
        if h[3] <= string_address and string_address + string_size <= h[3] + h[5]:
            begin = h[2] + string_address - h[3]
            strings = raw[begin:begin + string_size]
            break
    if strings is None:
        raise ValueError("ELF dynamic strings are outside LOAD data")
    imports = []
    for index in needed:
        if index >= len(strings) or b"\0" not in strings[index:]:
            raise ValueError("Invalid ELF import name")
        name = strings[index:strings.index(b"\0", index)].decode("ascii")
        if not name or "/" in name:
            raise ValueError("Invalid ELF import path")
        imports.append(name)
    return {"abi": abi, "load_alignment": [h[7] for h in loads], "imports": imports}


def inspect_apk_libraries(apk, abi="arm64-v8a"):
    """Check every bundled .so, its imports, and its actual ZIP data offset."""
    result = {}
    with zipfile.ZipFile(apk) as archive, open(apk, "rb") as stream:
        names = archive.namelist()
        if len(names) != len(set(names)):
            raise ValueError("APK contains duplicate entries")
        for entry in archive.infolist():
            if not entry.filename.endswith(".so"):
                continue
            parts = entry.filename.split("/")
            if len(parts) != 3 or parts[:2] != ["lib", abi]:
                raise ValueError("APK contains an unexpected native library location/ABI")
            if entry.compress_type != zipfile.ZIP_STORED:
                raise ValueError("Native libraries must be uncompressed")
            stream.seek(entry.header_offset)
            header = stream.read(30)
            if len(header) != 30 or header[:4] != b"PK\x03\x04":
                raise ValueError("Invalid ZIP local header")
            name_size, extra_size = struct.unpack_from("<HH", header, 26)
            data_offset = entry.header_offset + 30 + name_size + extra_size
            if data_offset % 16384:
                raise ValueError(f"{entry.filename}: APK data is not 16 KB aligned")
            identity = elf_identity(archive.read(entry),abi)
            identity["zip_data_offset"] = data_offset
            result[parts[2]] = identity
    if "libtbg_client.so" not in result:
        raise ValueError("APK has no ARM64 game library")
    for name, identity in result.items():
        missing = set(identity["imports"]) - SYSTEM_LIBRARIES - result.keys()
        if missing:
            raise ValueError(f"{name}: unbundled imports: {sorted(missing)}")
    return result
