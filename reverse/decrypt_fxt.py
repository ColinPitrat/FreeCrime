#!/usr/bin/python
# -*- coding: utf8 -*-"
import argparse
import sys

def decrypt(input_path, output_path, text_mode):
    try:
        with open(input_path, 'rb') as f:
            data = bytearray(f.read())

        # Decrypt first 8 bytes
        # "For every byte substract a value returned from left shifting 99"
        # We assume this means shifting 99 by the byte index (0-7)
        for i in range(min(8, len(data))):
            shift_val = (99 << i)
            # Python handles large ints, so we subtract and mask to get the byte
            data[i] = (data[i] - shift_val - 1) & 0xFF

        # Decrypt remaining bytes
        # "To get text, just substract 1 from every byte"
        for i in range(8, len(data)):
            data[i] = (data[i] - 1) & 0xFF
            if text_mode and data[i] == 0:
                data[i] = ord('\n')

        with open(output_path, 'wb') as f:
            f.write(data)

        print(f"Successfully decrypted '{input_path}' to '{output_path}'")

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Decrypt GTA FXT files.")
    parser.add_argument("input_file", help="Path to the .FXT file")
    parser.add_argument("output_file", help="Path to the decoded file to create")
    parser.add_argument("--text", "-t", action='store_true', help="If set, create a text file instead of simply decrypting (i.e. use line feed instead of NUL char)")

    args = parser.parse_args()

    decrypt(args.input_file, args.output_file, args.text)
