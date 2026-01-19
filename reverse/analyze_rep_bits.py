import struct
import sys

def analyze_bits(filenames):
    # bit_counts[byte_index][bit_index]
    bit_counts = [[0] * 8 for _ in range(4)]
    total_records = 0

    for filename in filenames:
        with open(filename, 'rb') as f:
            data = f.read()
            num_records = len(data) // 8
            total_records += num_records
            for i in range(num_records):
                record = data[i*8 + 4 : i*8 + 8]
                for b_idx in range(4):
                    byte_val = record[b_idx]
                    for bit_idx in range(8):
                        if (byte_val >> bit_idx) & 1:
                            bit_counts[b_idx][bit_idx] += 1

    print(f"Total records analyzed: {total_records}")
    for b_idx in range(4):
        print(f"\nByte {b_idx + 5} Bit Distribution (0-7):")
        used_bits = []
        unused_bits = []
        for bit_idx in range(8):
            count = bit_counts[b_idx][bit_idx]
            if count > 0:
                used_bits.append(bit_idx)
                print(f"  Bit {bit_idx}: SET in {count} records ({100*count/total_records:.2f}%)")
            else:
                unused_bits.append(bit_idx)
        
        print(f"  Summary: Bits {used_bits} are USED. Bits {unused_bits} are NEVER SET.")

if __name__ == "__main__":
    analyze_bits(sys.argv[1:])
