# FreeCrime

FreeCrime is a clone of [GTA 1](https://en.wikipedia.org/wiki/Grand_Theft_Auto_(video_game))
(Grand Theft Auto) written in [Rust](https://en.wikipedia.org/wiki/Rust_(programming_language))
using [Bevy](https://bevy.org/) with the objectives of:

 - reproducing the original game as faithfully as possible when provided the appropriate resources
   (including London 1969 and London 1961 mission packs)
 - support mods for the original games
 - provide a similar game experience with free/open source resources for players who don't have the
   original game
 - maybe, in the future, support the [GTA 2](https://en.wikipedia.org/wiki/Grand_Theft_Auto_2) game
   too.

## Current status

There is no code written, reverse engineering of the original game is on-going.
The on-going reverse engineering work is documented in the [reverse](reverse/README.md)
subdirectory.

## How to help?

You can help by playing the game! (How cool is that?)

### Identifying sprites

Currently, I'm trying to identify which sprite is used where.

The file [reverse/sprites.md](reverse/sprites.md) contains a list of identified
sprites. Some of them are uncertain, identifying where they are used in the game
would be helpful. Some are not listed, adding them would also be helpful.

It's not necessary to document the vehicles.

Explaining how/when the pedestrian sprites are used is useful.

You can extract sprites from the game's resources with:
```
python3 reverse/modify_gry.py --export resources_nyc24 ${PATH_TO_GTADATA:?}/STYLE001.G24
```

This will create a directory `resources_nyc24` in which you'll find a file
`STYLE001.G24.html`. Viewing this file in a browser will allow you to explore
the sprites.

### Documenting game mechanic

Identifying how some elements of the game are behaving / working will be very
helpful to reproduce it.

This can be as simple as:
 - standing at a corner and observing how pedestrian / cars are behaving
 - following pedestrian/vehicles and trying to understand how they behave
 - timing some events (e.g. how long does it take to fire 50 bullets)
 - verifying if some things are deterministic or random (e.g. does a particular
   car starting next to the player at game start take the same turn on the next
   intersection?)
 - measuring some vehicles properties (e.g. how many bullets before it explodes?
   how many tiles crossed at max speed? how long does it take to reach max
   speed? ...) and document them, so that they can later be compared to their
   properties in the game's resources

Note: The tool [modify_cmp.py](reverse/modify_cmp.py) can be useful to enable
some tests. For example, one can create a large are of road tiles to allow
testing cars.

TODO: Document how to use `modify_cmp.py` to create a large empty car park.
