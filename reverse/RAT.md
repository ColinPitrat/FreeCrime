# GTA 1 RAT file format

The RAT file format stores 8-bit indexed raw image data.

## File format

The RAT file does not contain any header. It is a raw sequence of bytes, where each byte is an index into a palette (stored in a separate file).

| name | size | notes |
|------|------|-------|
| image_data | width * height | 1 byte per pixel, indexed color |

The dimensions of the image (width and height) are not stored in the file and must be known beforehand or guessed. A common resolution used in the game is 640x480 (307,200 bytes).

To display the image, a palette must be loaded from a corresponding `.ACT` file (e.g., if the file is `IMAGE.RAT`, the palette is in `IMAGE.ACT`). If absent, the default palette from the file `F_PAL.RAW` is used.
