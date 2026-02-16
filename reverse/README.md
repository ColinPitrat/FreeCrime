## File formats

 - [ACT](ACT.md) - Palette for 8 bits raw graphics (.RAT)
 - [CMP](CMP.md) - Map file
 - [DAT](DAT.md) - Saved state and configuration (resolution, volume, language, highscores, etc...)
 - [FON](FON.md) - Font file, also used for sequences of images with constant height
 - [FXT](FXT.md) - Text file
 - [G24](G24.md) - Style for 24-bits version (needs more work)
 - [GRY](GRY.md) - Style for 8-bits version (almost finished)
 - [MISSION.INI](MISSION.INI.md) - Mission data, objects placement, etc...
 - [RAT](RAT.md) - Raw graphics in 8 bits (palette in .ACT)
 - [RAW](RAW.md) - Raw graphics in 24 bits
 - [REP](REP.md) - Replay?
 - [SDT](SDT.md) - Sound index (goes in pair with a RAW file for the sound data)

## Game information
 - [all_supported_keywords.txt](all_supported_keywords.txt) - All the supported MISSION.INI keywords, extracted from the executable with [extract_exe_strings.py](extract_exe_strings.py)
 - [all_used_keywords.txt](all_used_keywords.txt) - All the keywords actually used in MISSION.INI (combining both the original game and the London extension pack).
 - [game_mechanic.md](game_mechanic.md) - Some findings about the game mechanic.
 - [sprites.md](sprites.md) - Some findings about sprites.

## Reverse engineering tools

The main script is:
 - [display_map.py](display_map.py) is the main tool to reproduce game logic and investigate game mechanics. It can display a map, objects and has a play mode to test various things. The objective is to implement most of the game mechanic in it to serve as a reference implementation as well as a playground.

Helper scripts are:
 - [analyze_rep.py](analyze_rep.py) and [analyze_rep_bits.py](analyze_rep_bits.py) are useful to investigate the REP file format.
 - [decode_cmp.py](decode_cmp.py) and [modify_cmp.py](modify_cmp.py) are useful to investigate the CMP file format.
 - [decrypt_fxt.py](decrypt_fxt.py) can extract readable text from the FXT file format.
 - [display_fon.py](display_fon.py) can display graphics from a FON file.
 - [display_raw.py](display_raw.py) can display graphics from a RAW file and is useful to help determine its size.
 - [extract_exe_strings.py](extract_exe_strings.py) can extract a list of strings from the executable of the game. It was used in particular to extract all supported keywords for the MISSION.INI file.
 - [extract_sounds.py](extract_sounds.py) can extract sounds from SDT and RAW files.
 - [modify_dat.py](modify_dat.py) is useful to investigate the DAT file format.
 - [modify_gry.py](modify_gry.py) is useful to investigate the GRY and G24 file formats.
