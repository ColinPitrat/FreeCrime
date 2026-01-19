#!/usr/bin/python
# -*- coding: utf8 -*-"
import argparse
import struct
import os
import wave

def extract_sounds(sdt_path, output_dir, channels, bits):
    """
    Extracts sounds from a .RAW file using a .SDT index file.
    """
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    raw_path = sdt_path.replace('SDT', 'RAW').replace('sdt', 'raw')
    filename = os.path.basename(sdt_path)[:-4]

    print(f"Extracting from {sdt_path} and {raw_path} to {output_dir}/{filename}*")

    try:
        with open(sdt_path, 'rb') as sdt_file, open(raw_path, 'rb') as raw_file:
            index = 0
            while True:
                # Read 12 bytes (3 * 4 bytes integers)
                # <offset>, <size>, <frequency>
                entry_data = sdt_file.read(12)
                if len(entry_data) < 12:
                    break

                # Assuming Little Endian (<) unsigned int (I)
                offset, size, frequency = struct.unpack('<III', entry_data)

                # Read raw audio data
                raw_file.seek(offset)
                audio_data = raw_file.read(size)

                if len(audio_data) != size:
                    print(f"Warning: Entry {index} expected {size} bytes but got {len(audio_data)}")

                # Output filename
                out_filename = os.path.join(output_dir, f"{filename}_{index:04d}.wav")

                try:
                    with wave.open(out_filename, 'wb') as wav_file:
                        wav_file.setnchannels(channels)
                        wav_file.setsampwidth(bits // 8)
                        wav_file.setframerate(frequency)
                        wav_file.writeframes(audio_data)
                    print(f"Extracted: {out_filename} (Freq: {frequency}Hz, Size: {size})")
                except Exception as e:
                    print(f"Error writing {out_filename}: {e}")

                index += 1

    except FileNotFoundError as e:
        print(f"Error: {e}")
    except Exception as e:
        print(f"An unexpected error occurred: {e}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Extract audio from GTA .RAW/.SDT files.")
    parser.add_argument("sdt_file", help="Path to the .SDT file")
    parser.add_argument("--out", "-o", default="extracted_sounds", help="Output directory")
    parser.add_argument("--channels", "-c", type=int, choices=[1, 2], default=1, help="Number of channels (1=Mono, 2=Stereo)")
    parser.add_argument("--bits", "-b", type=int, choices=[8, 16], default=8, help="Bit depth (8 or 16)")

    args = parser.parse_args()

    extract_sounds(args.sdt_file, args.out, args.channels, args.bits)
