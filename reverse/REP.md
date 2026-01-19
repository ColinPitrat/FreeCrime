The REP files are very likely replay files that allow the game to display the
demo when it remains idle for some time (TODO: check after how long it triggers).

DISCLAIMER: Their format is not documented and so far I didn't test much. What
follows is only idea I got from looking at the files content.

All the REP files have a size multiple of 8 bytes. They also tend to be
relatively small. It would make sense that such a file only stores keystrokes to
reproduce the movements and actions of the player. Looking at the hexdump of the
files, there seems to be a pattern which follows a 4, 8 or 16 bytes alignment.

Considering the file size, the data to be stored (timing + key strokes) and the
look of the file, I strongly suspect it's a list of 8 bytes record.

The first 4 bytes (or 2 bytes?) in little endian are increasing. I suspect they
are a timing after which to press a key. Considering other GTA considerations,
it would make sense to be a number of frames (25 per second).

The next 4 bytes would then, somehow, represent the keystrokes. The 8th bytes is
almost always 0 except for the last 2 records (always 40 and then 0E). These
likely indicate the end of the demo (command to exit the game?).

I have one REPLAY.REP file (not sure where it comes from) where the 8th bytes
sometimes takes non-zero value outside of the two last records. The values
taken are: 15 1B 55 5B A4 A8 E4 E8 which in binary are:
 - 00010101
 - 00011011
 - 01010101
 - 01011011
 - 10100100
 - 10101000
 - 11100100
 - 11101000

Not forgetting 40 and 0E which are:
 - 01000000
 - 00001110

Bit Usage Summary

┌────────┬────────────────────────┬─────────────┬─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Byte   │ Used Bits              │ Unused Bits │ Key Observations                                                                                                    │
├────────┼────────────────────────┼─────────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│ Byte 5 │ 0, 1, 2, 3, 6, 7       │ 4, 5        │ Bits 0 and 1 are extremely common (~50% each), likely representing a primary state like "On Foot" vs. "In Vehicle". │
│ Byte 6 │ 1, 2, 3, 4, 5, 6, 7    │ 0           │ Most bits are used. Bit 0 is never set. High activity on bits 1 and 7.                                              │
│ Byte 7 │ 0, 4, 5, 6, 7          │ 1, 2, 3     │ Bit 0 is used in exactly the same number of records as Byte 6's Bit 7 (1484 records), suggesting they are linked.   │
│ Byte 8 │ 0, 1, 2, 3, 4, 5, 6, 7 │ (None)      │ Every bit is used at least once, though they are overall very rare (~0.3% - 0.5%).                                  │
└────────┴────────────────────────┴─────────────┴─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

Potential Significance
- Correlations: The perfect match between Byte 6/Bit 7 and Byte 7/Bit 0 (both 23.64%) suggests a 16-bit field spanning these two bytes, or a specific event type that triggers both.
- Byte 5/6 Activity: These appear to be the primary input/state bytes. The "toggle" behavior I noticed earlier (values returning to 0) is most prevalent in Byte 6, reinforcing the idea that it tracks key
 presses/releases.
- Unused Bits: Bits 4 and 5 in Byte 5, bit 0 in Byte 6, and bits 1, 2, and 3 in Byte 7 are remarkably consistent in being unused across thousands of records, which helps narrow down the possible data
 structures.

Tentative structure:
   - Each record is 8 bytes:
       - timestamp/frame (4 bytes, Little Endian)
       - 4 bytes encoding actions
   - From the patterns observed:
       - Byte 5: Frequently takes values like 1 and 2.
       - Byte 6: Shows "on/off" patterns (e.g., 14 or 18 followed by 0), likely representing specific keys like accelerate, brake, or shoot.
       - Byte 7: Less frequent, but contains values like 1, 16, and 64, possibly for less common actions or system-level events.
       - Byte 8: Mostly 0 in smaller replays, but contains various values in REPLAY.REP, potentially related to more complex state changes.

TODO: Record the demo and analyze it in slow motion.

TODO: Modify the files one byte at a time and see what changes.
