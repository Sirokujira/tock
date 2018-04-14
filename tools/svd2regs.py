# usage: svd2regs.py [-h] (--mcu VENDOR MCU | --svd SVD) [--save FILE] [--fmt]
# [--path PATH] [--args ARGS]
# [PERIPHERAL [PERIPHERAL ...]]
#
# positional arguments:
# PERIPHERAL        Name of the Peripheral
#
# optional arguments:
# -h, --help        show this help message and exit
# --mcu VENDOR MCU  Vendor and MCU (Database from cmsis-svd)
# --svd SVD         Path to SVD-File
# --save FILE       Save generated Code to file
#
# rustfmt:
# Format with rustfmt
#
# --fmt ['ARG ..']  enable rustfmt with optional arguments
# --path PATH       path to rustfmt
#
# Examples:
#   SIM peripheral from Database
#     svd2regs.py SIM --mcu Freescale MK64F12
#
#   Format with rustfmt
#     svd2regs.py SIM --svd mcu.svd --fmt
#
#   Format with rustfmt --force
#     svd2regs.py SIM --svd mcu.svd --fmt '--force'
#
#   Format with rustfmt not in PATH
#     svd2regs.py SIM --svd mcu.svd --fmt --path /home/tock/bin/
#
#   Save to file
#     svd2regs.py SIM --svd mcu.svd --fmt '--force' --save src/peripherals.rs
#
#   With stdin pipe
#     cat mcu.svd | svd2regs.py SIM --svd --fmt '--force' | tee src/mcu.rs
#
# Required Python Packages:
#   cmsis-svd
#   pydentifier
#
# Author: Stefan Hoelzl <stefan.hoelzl@posteo.de>

import sys
import argparse
from subprocess import Popen, PIPE
from xml.etree import ElementTree as ET

from cmsis_svd.parser import SVDParser
import pydentifier

RUST_KEYWORDS = ["mod"]
COMMENT_MAX_LENGTH = 80


def comment(text):
    return "/// {}".format(text[:COMMENT_MAX_LENGTH].strip())


class CodeBlock(str):
    TEMPLATE = ""

    def __new__(cls, obj=None):
        return cls.TEMPLATE.format(**cls.fields(obj))

    @staticmethod
    def fields(obj):
        return {}


class Includes(CodeBlock):
    TEMPLATE = """
use kernel::StaticRef;
use kernel::common::regs::{{self, ReadOnly, ReadWrite, WriteOnly}};
    """


class PeripheralBaseDeclaration(CodeBlock):
    TEMPLATE = """
const {name}_BASE: StaticRef<{title}Registers> =
    unsafe {{ StaticRef::new(0x{base:8X} as *const {title}Registers) }};
"""

    @staticmethod
    def fields(peripheral):
        return {
            "name": peripheral.name,
            "title": peripheral.name.title(),
            "base": peripheral.base_address,
        }


class PeripheralStruct(CodeBlock):
    TEMPLATE = """{comment}
#[repr(C, packed)]
struct {name}Registers {{
{fields}
}}
"""

    @staticmethod
    def fields(peripheral):
        return {
            "comment": comment(peripheral.description),
            "name": peripheral.name.title(),
            "fields": "\n".join(
                PeripheralStructField(register)
                for register in peripheral.registers
            )
        }


class PeripheralStructField(CodeBlock):
    TEMPLATE = """{comment}
{name}: {mode}<u{size}{definition}>,"""

    @staticmethod
    def fields(register):
        mode_map = {
            "read-only": "ReadOnly",
            "read-write": "ReadWrite",
            "write-only": "WriteOnly",
        }

        def identifier(name):
            identifier =  name.lower()
            if identifier in RUST_KEYWORDS:
                identifier = "{}_".format(identifier)
            return identifier

        def definition(reg):
            if len(reg._fields) == 1:
                return ""
            return ", {}::Register".format(reg.name)

        return {
            "comment": comment(register.description),
            "name": identifier(register.name),
            "size": register._size,
            "mode": mode_map[register._access],
            "definition": definition(register),
        }


class BitfieldsMacro(CodeBlock):
    TEMPLATE = """register_bitfields![u{size},{bitfields}
];"""

    @staticmethod
    def fields(registers):
        bitfields = ",".join(Bitfield(register) for register in registers)
        return {
            "size": 32,
            "bitfields": bitfields
        }


class Bitfield(CodeBlock):
    TEMPLATE = """
{name} [
{fields}
]"""

    @staticmethod
    def fields(register):
        fields = ",\n".join(BitfieldField(field) for field in register._fields)
        return {
            "name": register.name,
            "fields": fields,
        }


class BitfieldField(CodeBlock):
    TEMPLATE = """    {comment}
    {name} OFFSET({offset}) NUMBITS({size}) {enums}"""

    @staticmethod
    def enumerated_values(field):
        values = []
        for value in field.enumerated_values:
            if value.description not in [v.description for v in values]:
                values.append(value)
        return values

    @staticmethod
    def fields(field):
        if not field.is_enumerated_type:
            enums = "[]"
        else:
            enums = ",\n".join(BitfieldFieldEnum(enum)
                               for enum in BitfieldField.enumerated_values(field))
            enums = "[\n{}\n    ]".format(enums)
        return {
            "comment": comment(field.description),
            "name": field.name,
            "offset": field.bit_offset,
            "size": field.bit_width,
            "enums": enums,
        }


class BitfieldFieldEnum(CodeBlock):
    TEMPLATE = """        {comment}
        {name} = {value}"""

    @staticmethod
    def fields(enum):
        def identifier(desc):
            if any(desc.startswith(str(digit)) for digit in range(10)):
                desc = "_{}".format(desc)
            i = pydentifier.upper_camel(desc)
            return i if len(i) < 80 else None

        def enum_identifier(e):
            for t in [e.description, e.name, e.value]:
                i = identifier(t)
                if i:
                    return i

        return {
            "comment": comment(enum.description),
            "name": enum_identifier(enum),
            "value": enum.value,
        }


def get_parser(mcu, svd):
    try:
        if mcu:
            return SVDParser.for_packaged_svd(mcu[0], "{}.svd".format(mcu[1]))
        return SVDParser(ET.fromstring(svd.read()))
    except IOError:
        print("No SVD file found")
        sys.exit()


def parse(peripherals, mcu, svd):
    svd_parser = get_parser(mcu, svd)
    dev = svd_parser.get_device()
    if not peripherals:
        peripherals = [peripheral.name for peripheral in dev.peripherals]
    return filter(lambda p: p.name in peripherals, dev.peripherals)


def generate(peripherals):
    return Includes() + "\n".join(generate_peripherial(p) for p in peripherals)


def generate_peripherial(peripheral):
    return generate_peripheral_struct(peripheral) \
           + generate_bitfields_macro(filter(lambda r: len(r._fields) > 1,
                                      peripheral.registers)) \
           + PeripheralBaseDeclaration(peripheral)


def generate_peripheral_struct(peripheral):
    return PeripheralStruct(peripheral)


def generate_bitfields_macro(registers):
    return BitfieldsMacro(registers)


def rustfmt(code, path, *args):
    cmd = ["{}rustfmt".format(path)]
    cmd.extend(args)
    fmt = Popen(cmd, stdin=PIPE, stdout=PIPE, stderr=PIPE)
    out, err = fmt.communicate(code)
    if err:
        print code
        print err
        sys.exit()

    return out


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("peripherals", nargs="*", metavar="PERIPHERAL",
                        help="Name of the Peripheral")
    xor = parser.add_mutually_exclusive_group(required=True)
    xor.add_argument('--mcu', nargs=2, metavar=('VENDOR', 'MCU'),
                     help='Vendor and MCU (Database from cmsis-svd)')
    xor.add_argument('--svd', type=argparse.FileType('r'), default=sys.stdin,
                     help='Path to SVD-File')
    parser.add_argument("--save", type=argparse.FileType('w'), metavar="FILE",
                        default=sys.stdout, help="Save generated Code to file")
    fmt = parser.add_argument_group('rustfmt',
                                    'Format with rustfmt')
    fmt.add_argument("--fmt", nargs="?", const='', metavar="'ARG ..'",
                     help="enable rustfmt with optional arguments")
    fmt.add_argument("--path", help="path to rustfmt", default="")
    return parser.parse_args()


def main():
    args = parse_args()
    code = generate(parse(args.peripherals, args.mcu, args.svd))
    if args.fmt is not None:
        code = rustfmt(code, args.path, *args.fmt.strip("'").split(" "))
    args.save.write(code)


if __name__ == '__main__':
    main()
