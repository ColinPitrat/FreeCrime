# GTA 1 RAW file format

The RAW file format is used for raw data, typically representing either 24-bit RGB images.
An exception is `F_PAL.RAW` which stores a 256-color palettes (a tell-tale is the 768 bytes size).

## File format

The RAW file does not contain any header. Its content interpretation depends on its size and context.

### 24-bit RGB Image

In most cases, a RAW file represents a 24-bit RGB image.

| name | size | notes |
|------|------|-------|
| image_data | width * height * 3 | 3 bytes per pixel (Red, Green, Blue) |

The dimensions of the image (width and height) are not stored in the file. For a 640x480 resolution, the file size is 921,600 bytes.
The values for color components are between 0 and 255.

### Palette Data

Some RAW files, notably `F_PAL.RAW`, store 256-color palette data.

| name | size | notes |
|------|------|-------|
| palette | 768 | 256 colors, 3 bytes per color (RGB) |

The values for color components are between 0 and 255.

This is identical to the [ACT](ACT.md) file format but uses a different extension. These files are used as palettes for [RAT](RAT.md) images.
