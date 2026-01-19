import struct
import sys
import os
from collections import Counter

def analyze_rep(filename):
    print(f"--- Analyzing {filename} ---")
    with open(filename, 'rb') as f:
        data = f.read()
    
    num_records = len(data) // 8
    records = []
    for i in range(num_records):
        records.append(struct.unpack('<I BB BB', data[i*8:(i+1)*8]))
    
    # Check if first 4 bytes (timestamp/frame) are increasing
    timestamps = [r[0] for r in records]
    increasing = all(timestamps[i] <= timestamps[i+1] for i in range(len(timestamps)-1))
    print(f"Increasing timestamps: {increasing}")
    if len(timestamps) > 1:
        print(f"Start: {timestamps[0]}, End: {timestamps[-1]}, Avg Delta: {(timestamps[-1]-timestamps[0])/(len(timestamps)-1):.2f}")

    # Analyze bytes 5, 6, 7, 8 (indices 1, 2, 3, 4 in our unpack)
    for b_idx in range(1, 5):
        vals = [r[b_idx] for r in records]
        counts = Counter(vals)
        print(f"Byte {b_idx+4} unique values: {sorted(counts.keys())}")
        if len(counts) < 20:
            print(f"  Distribution: {dict(counts)}")

    # Look for patterns (toggle 0/non-zero)
    print("Toggle patterns (byte: value -> 0):")
    for b_idx in range(1, 5):
        toggles = 0
        for i in range(len(records)-1):
            if records[i][b_idx] != 0 and records[i+1][b_idx] == 0:
                toggles += 1
        print(f"  Byte {b_idx+4} toggles: {toggles}")

if __name__ == "__main__":
    for arg in sys.argv[1:]:
        analyze_rep(arg)
