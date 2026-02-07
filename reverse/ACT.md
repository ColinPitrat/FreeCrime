# GTA 1 ACT file format

The ACT file format stores palette information for 8-bit images (RAT files).

## File format

| name | size | notes |
|------|------|-------|
| palette | 768 bytes | 256 colors, 3 bytes per color (RGB) |

The palette contains 256 colors. Each color is stored as 3 consecutive bytes representing the Red, Green, and Blue components respectively.
The values for color components are between 0 and 255.

ACT files are usually associated with a RAT file of the same name (e.g., `CUT0.RAT` and `CUT0.ACT`).
