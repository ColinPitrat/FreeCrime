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

Reverse engineering of the original game is still on-going (and will likely
never be fully finished!).

There is a map viewer which allows to explore the map in 3D implemented already.

## Usage
```bash
# Show summary of a file
cargo run -- info gamedata/gta/NYC.CMP

# Extract content (text from FXT, BMPs from FON/GRY)
cargo run -- extract gamedata/gta/ENGLISH.FXT english.txt

# Generate a map overview BMP
cargo run -- overview gamedata/gta/NYC.CMP

# Interactive 3D map viewer (Bevy)
cargo run -- display gamedata/gta/NYC.CMP gamedata/gta/STYLE001.GRY
```

Supported extraction:
- **FXT**: Decrypts and saves to a plain text file.
- **FON**: Extracts all glyphs as 32-bit BMP images.
- **GRY**: Extracts block faces as 32-bit BMP images.
- **SDT**: Exports sound index metadata as a CSV file.

The `overview` command generates a static top-down overview of the map as a `map_overview.bmp` file.

The `display` command launches an interactive 3D viewer using Bevy.
- **Controls**:
  - **WASD**: Move horizontally
  - **Space/Shift**: Move Up/Down
  - **Q/E**: Rotate camera

## How to help?

You can help by playing the original game! (How cool is that?)

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

To create a car testing area in Liberty City, in the starting place of the first
level, start by creating a copy of the map (you're going to mess with it):

```
cp ${PATH_TO_GTADATA:?}/NYC.CMP ${PATH_TO_GTADATA:?}/NYC.CMP.ORI
```

Beware: always keep a backup of the game's original resources somewhere safe.
There's always a risk of messing things up with an incorrect command, even when
doing a manual copy before each modification.

Then use `modify_cmp.py` to copy the tile `7,5` around the starting area:

```
python3 reverse/modify_cmp.py ${PATH_TO_GTADATA:?}/NYC.CMP --generate 7,5 --output ${PATH_TO_GTADATA:?}/NYC.CMP
```

When you're done, you can restore the original map with:

```
cp ${PATH_TO_GTADATA:?}/NYC.CMP.ORI ${PATH_TO_GTADATA:?}/NYC.CMP
```
