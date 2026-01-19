SDT files are storing game sounds, including background music and sound effects.
It's a proprietary file format
Some programs, like [Dragon UnPACKer](https://github.com/elbereth/DragonUnPACKer.git) and MAStudio 2002, can open and copy the audio content from SDT files.

It seems to be an index on the corresponding RAW file.

== Digging into the sounds using GRE

Using [GRE](https://www.gtagarage.com/mods/show.php?id=1567) it seems that:
 - LEVEL000 contains 10 sounds
 - LEVEL00[1-3] contain 79 sounds each
Though listening at the sounds, it's unclear whether they are actually properly cut.

Duration of the sounds in LEVEL000 are:
 - 2s
 - 1s
 - 0.8s
 - 5s (seems to contain multiple sounds)
 - 0.7s
 - 49s (seems to contain multiple sounds)
 - 0s (invalid?)
 - 0.3s
The frequencies also seem off:
Sound0.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 32000 Hz
Sound1.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 135136 Hz
Sound2.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 499888 Hz
Sound3.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 30000 Hz
Sound4.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 59720 Hz
Sound5.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 823376 Hz
Sound6.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 30000 Hz
Sound7.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 9598 Hz
Sound8.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 980434 Hz
Sound9.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 16 bit, mono 44100 Hz

Still, taking this as a basis and looking at files sizes of LEVEL00\*.SDT files:
  192 ../../GTA/gtadata/AUDIO/LEVEL000.SDT
 1572 ../../GTA/gtadata/AUDIO/LEVEL001.SDT
 1572 ../../GTA/gtadata/AUDIO/LEVEL002.SDT
 1572 ../../GTA/gtadata/AUDIO/LEVEL003.SDT

Guessing a linear relation for the file size in terms of number of sounds, we get:
 10*a + b = 192
 79*a + b = 1572

Leading to:
 a = 20
 b = -8

Which also suggest something is off as `b` is negative!

Doing a diff on the LEVEL001.SDT, LEVEL002.SDT and LEVEL003.SDT we can see that the 3 files have an identical start.
The difference only starts around offset `0x4b0` suggesting most of the sounds are shared between the levels and only a few are different.

Similarly, LEVEL001.RAW, LEVEL002.RAW and LEVEL003.RAW have an identical start up to offset `0xd5e00` (out of total sizes of `0xfef99`, `0xfacc6` and `0xfd48c`).

Looking at the hexdump for LEVEL001.SDT, we can see 4 bytes numbers, most of which fit on 2 bytes. The file does contain a few numbers that require 3 bytes, which happen regularly and increase monotonically. The last of them is `0xf8075` (compatible with an offset in the RAW file which has length `0xfef99`) and is followed by `0x6f24` which happens to be exactly the length from `0xf8075` to the end of the file. It is followed by `0x2b11` which is 11025 and could very well be the frequency of the sound.

Going back in the file, it seems there's no other data than that and therefore, the file format would be a list of: `<offset> <size> <frequency>` with each number being on 4 bytes in little endian.

The surprising part is that this leads to a large number of different frequencies, some of which are quite unusual:
 - a0 0f ->  4,000 Hz
 - 88 13 ->  5,000 Hz
 - 7c 15 ->  5,500 Hz
 - 70 17 ->  6,000 Hz
 - 40 1f ->  8,000 Hz
 - 11 2b -> 11,025 Hz
 - 20 35 -> 13,600 Hz
 - b0 36 -> 14,000 Hz
 - 5e 39 -> 14,686 Hz
 - d0 39 -> 14,800 Hz
 - 34 3a -> 14,900 Hz
 - 98 3a -> 15,000 Hz
 - 60 3b -> 15,200 Hz
 - 80 3e -> 16,000 Hz
 - 68 42 -> 17,000 Hz
 - 38 4a -> 19,000 Hz
 - 22 56 -> 22,050 Hz

A small python script to test this shows that extracting single channel 16-bits data from the RAW file using these parameters works well for LEVEL000.
However, for all other files, 8-bits data is required. There doesn't seem to be anything indicating that in the SDT file, which suggests that it is either to be found in the RAW file or needs to be known before reading the data (in the code itself, or maybe from some other file).
In the RAW file, a clear difference is that the first byte of LEVEL000 is `0` whereas the other files have a value close to `0x80`.
