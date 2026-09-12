"""Elevate the outer loader without moving Inno Setup's appended payload."""
import re
import struct
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


def manifest(data):
    pe = struct.unpack_from('<I', data, 60)[0]
    if data[pe:pe + 4] != b'PE\0\0':
        raise ValueError('Not a PE executable')
    count = struct.unpack_from('<H', data, pe + 6)[0]
    optional = pe + 24
    optional_size = struct.unpack_from('<H', data, pe + 20)[0]
    magic = struct.unpack_from('<H', data, optional)[0]
    directories = optional + {0x10b: 96, 0x20b: 112}[magic]
    if struct.unpack_from('<I', data, directories + 4 * 8 + 4)[0]:
        raise ValueError('Apply elevation before signing the executable')

    def offset(rva):
        for index in range(count):
            section = optional + optional_size + 40 * index
            virtual_size, address, raw_size, raw = struct.unpack_from('<IIII', data, section + 8)
            if address <= rva < address + min(virtual_size, raw_size):
                return raw + rva - address
        raise ValueError('Resource address is outside the PE sections')

    base = offset(struct.unpack_from('<I', data, directories + 2 * 8)[0])

    def entries(relative):
        named, numbered = struct.unpack_from('<HH', data, base + relative + 12)
        return dict(struct.unpack_from('<II', data, base + relative + 16 + 8 * i)
                    for i in range(named + numbered))

    directory = 0
    for resource_id in (24, 1):  # RT_MANIFEST, CREATEPROCESS_MANIFEST_RESOURCE_ID
        child = entries(directory)[resource_id]
        if not child & 0x80000000:
            raise ValueError('Invalid manifest resource directory')
        directory = child & 0x7fffffff
    languages = entries(directory)
    if len(languages) != 1:
        raise ValueError('Expected one manifest language')
    entry = next(iter(languages.values()))
    if entry & 0x80000000:
        raise ValueError('Expected manifest resource data')
    rva, size = struct.unpack_from('<II', data, base + entry)
    start = offset(rva)
    text = data[start:start + size].decode('utf-8-sig').rstrip('\0')
    root = ET.fromstring(text)
    levels = list(root.iter('{urn:schemas-microsoft-com:asm.v3}requestedExecutionLevel'))
    if len(levels) != 1:
        raise ValueError('Expected exactly one requestedExecutionLevel')
    return start, size, text, levels[0].attrib


def elevate(installer, application):
    if manifest(application.read_bytes())[3].get('level') != 'asInvoker':
        raise ValueError('Main application must remain asInvoker')
    original = installer.read_bytes()
    start, size, text, attributes = manifest(original)
    if attributes.get('level') != 'asInvoker' or attributes.get('uiAccess') != 'false':
        raise ValueError('Expected an unsigned asInvoker loader with uiAccess=false')
    text, replacements = re.subn(r'''level=(['"])asInvoker\1''',
                                  'level="requireAdministrator"', text)
    if replacements != 1:
        raise ValueError('Expected one loader execution level to replace')
    # Reclaim XML formatting whitespace only; preserve all other manifest settings.
    compact = re.sub(r'>\s+<', '><', text).strip().encode('utf-8')
    if len(compact) > size:
        raise ValueError('Elevated manifest does not fit the existing resource')
    result = original[:start] + compact.ljust(size, b' ') + original[start + size:]
    if len(result) != len(original) or manifest(result)[3].get('level') != 'requireAdministrator':
        raise ValueError('Manifest verification failed')
    installer.write_bytes(result)
    if installer.read_bytes() != result:
        raise ValueError('Installer write verification failed')
    print('Verified: installer=requireAdministrator; application=asInvoker.')
    print('Only the manifest resource changed; file length and appended payload are unchanged.')


if __name__ == '__main__':
    if len(sys.argv) != 3:
        raise SystemExit('Usage: set-installer-elevation.py INSTALLER.exe APPLICATION.exe')
    elevate(Path(sys.argv[1]), Path(sys.argv[2]))
