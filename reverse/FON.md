# GTA 1 FON file format

The "FON" files are files that contain a sequence of images with a constant height.
They are used for Fonts and the moving parts in the cut scenes.

There structure is:

| name | size | notes |
|------|------|-------|
| `num_pictures` | byte | number of pictures in the file |
| height | byte | height in pixels of the pictures |
| pictures | dynamic (see below) | |
| palette | 768 bytes | 256 triplets (red, green, blue) |


The pictures are stored sequentially with the following format.

| name | size | notes |
|------|------|-------|
| width | byte | width of the picture |
| pixels | height\*width bytes | raw picture data (one byte per color) |

