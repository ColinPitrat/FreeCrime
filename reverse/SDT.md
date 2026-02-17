# GTA 1 SDT file format

SDT files are storing game sounds, including background music and sound effects.
It's an index on the corresponding RAW file.

## SDT file format

The SDT file format is a list of 12 bytes records, each record having the format
`<offset> <size> <frequency>` with each number being on 4 bytes in little endian.

These values refer to the data contained in the RAW file with the same name. The
RAW data uses a single channel. It can be either 16-bits (LEVEL000) or 8-bits
(all other files).

There doesn't seem to be anything indicating whether the data is 8-bits or
16-bits in the SDT file, which suggests that it is either to be found in the RAW
file or needs to be known before reading the data (in the code itself, or maybe
from some other file).

In the RAW file, a clear difference is that the first byte of LEVEL000 is `0`
whereas the other files have a value close to `0x80`. It seems unlikely that
this is how the bit depth is determined though.

Some sounds in LEVEL000 do not sound proper if decoding it with a single channel
with the frequency provided in the SDT. For example the sound 2 (played when
pressing Enter in the menu).
However, they sound better when decoded assuming 2 channels (stereo) or at
double the frequency.
Listening carefully to the game's playing this sound, it does seem to be stereo.

It seems the first 3 sounds in LEVEL000 are stereo.

However other sounds don't work properly if decoded in stereo and are apparently
mono (for example sound 6: "Music for your pleasure", played in Options menu
when selecting Music Mode: Radio).
