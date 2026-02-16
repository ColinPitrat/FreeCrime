#!/usr/bin/env python3
import argparse
import sys
import os

def main():
    parser = argparse.ArgumentParser(
        description="Extract null-terminated strings from a binary file starting at a given offset until multiple null bytes are encountered.",
        epilog=f"""Example:
  python3 {sys.argv[0]} GTA24.EXE 0x1e81cd
This will extract the keywords used in MISSION.INI from GTA24.EXE.""",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("file", help="The binary file to extract strings from")
    parser.add_argument("offset", help="The offset to start reading from (can be decimal or hex like 0x123)")

    args = parser.parse_args()

    try:
        offset = int(args.offset, 0)
    except ValueError:
        print(f"Error: Invalid offset '{args.offset}'. Must be an integer (decimal or hex).", file=sys.stderr)
        sys.exit(1)

    if not os.path.exists(args.file):
        print(f"Error: File '{args.file}' not found.", file=sys.stderr)
        sys.exit(1)

    try:
        with open(args.file, "rb") as f:
            f.seek(offset)
            current_string = b""
            i = 0
            while True:
                b = f.read(1)
                if b == b'\x00':
                    if current_string:
                        print(current_string.decode('ascii', errors='replace'))
                        current_string = b""
                    else:
                        break
                else:
                    current_string += b
    except Exception as e:
        print(f"Error reading file: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
