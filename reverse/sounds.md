# GTA 1 sounds

This page describes the sounds found in the files `LEVEL00*.SDT` and the
corresponding `LEVEL00*.RAW` files of the original GTA 1 game.

Interestingly, despite having a file per level, the sounds are actually almost
all identical between the 3 levels outside of the district names and a few
jingles.

All sounds are 8-bits mono unless specified otherwise.

## LEVEL000

All sounds in LEVEL000 are 16-bits.
Sounds 0, 1 and 2 are stereo. All the others are mono.

 - 0000: car passing (pressing left in levels selection menu)
 - 0001: car passing (pressing right in levels selection menu)
 - 0002: car passing + "all right" (pressing Enter in menu)
 - 0003: camera shutter (changing character in character selection menu)
 - 0004: car skidding (pressing Escape in menu)
 - 0005: recorder (selecting Music Mode: Constant in Options menu)
 - 0006: "Music for your pleasure" (selecting Music Mode: Radio in Options menu)
 - 0007: car engine (pressing up in menu)
 - 0008: car engine (pressing down in menu)
 - 0009: slow typewriter (Text: Slow in Options menu)
 - 0010: typewriter (Text: Normal in Options menu)
 - 0011: fast typewriter (Text: Fast in Options menu)
 - 0012: typewriter? (typing name in character selection menu)
 - 0013: typewriter? (pressing reset on character in character selection menu)
 - 0014: honk (increasing sound in Options)
 - 0015: short vinyl scrach? (usage unknown)

## LEVEL001

TODO: better identify all sounds: when are they played, how?

TODO: Understand/verify association of engine (in car info) and sound. Engine
      sounds seem to go from 45 to 57 which matches the range of 0-12 for the
      engine field in car info. Engines 7 and 8 are never used (corresponding
      to sounds 52 and 53)
      The frequency of the sound is likely increased more or less proportionally
      with the speed.

 - 0000: car door opening
 - 0001: car door closing
 - 0002: weird alien spaceship sound
 - 0003: weird alien spaceship sound
 - 0004: taking a bonus?
 - 0005: car engine starting
 - 0006: car hitting something?
 - 0007: window breaking?
 - 0008: something being shattered
 - 0009: something being shattered (more metallic)
 - 0010: explosion
 - 0011: hitting some bench or similar with a car
 - 0012: shooting a gun with a silencer???
 - 0013: a bunch of things falling down
 - 0014: ???
 - 0015: punching someone who moans
 - 0016: punch
 - 0017: punch with crack
 - 0018: falling into water
 - 0019: ???
 - 0020: ???
 - 0021: pedestrian shouting
 - 0022: pedestrian shouting
 - 0023: pedestrian shouting
 - 0024: pedestrian shouting (longer, more suffering, e.g. being crushed)
 - 0025: pedestrian shouting
 - 0026: pedestrian shouting (long)
 - 0027: laser?
 - 0028: phone ringing
 - 0029: someone talking on the radio (undistinguishable)
 - 0030: changing radio station
 - 0031: changing radio station
 - 0032: changing radio station
 - 0033: shooting gun
 - 0034: shooting gun
 - 0035: ???
 - 0036: weird shooting gun?
 - 0037: loading gun?
 - 0038: ???
 - 0039: explosion (long)
 - 0040: pager bip
 - 0041: ???
 - 0042: ???
 - 0043: bullet flying
 - 0044: weird zip sound?
 - 0045: engine
 - 0046: engine
 - 0047: engine (sports cars)
 - 0048: engine
 - 0049: engine
 - 0050: engine (bike, superbike, bug)
 - 0051: engine (bus, coach, fire truck, tanker, ...)
 - 0052: ???
 - 0053: ???
 - 0054: remote controlled car engine
 - 0055: train engine
 - 0056: tank engine
 - 0057: boat engine
 - 0058: honk
 - 0059: honk
 - 0060: honk
 - 0061: honk
 - 0062: honk
 - 0063: honk
 - 0064: car alarm
 - 0065: car skidding
 - 0066: police siren (TODO: doublecheck vs. ambulance siren)
 - 0067: ambulance siren
 - 0068: ???
 - 0069: bip (reversing alarm?)
 - 0070: bank alarm
 - 0071: ???
 - 0072: helicopter?
 - 0073: hare krishna
 - 0074: ???
 - 0075: fire?
 - 0076: burp
 - 0077: fart
 - 0078: ???
 - 0079: "We've got a "
 - 0080: "There's a "
 - 0081: "Proceed to a "
 - 0082: " and "
 - 0083: " zero "
 - 0084: " nine "
 - 0085: " twelve "
 - 0086: " fourteen "
 - 0087: " twenty-four "
 - 0088: " twenty-eight "
 - 0089: " thirty-two "
 - 0090: " thirty-four "
 - 0091: " thirty-five "
 - 0092: " forty-two "
 - 0093: " seventy-one "
 - 0094: " ninety "
 - 0095: " ninety-one "
 - 0096: " ninety-six "
 - 0097: " in " ???
 - 0098: nothing???
 - 0099: nothing???
 - 0100: "Liberty City"
 - 0101: "Hackenslash"
 - 0102: "Eaglewood"
 - 0103: "Guernsey city"
 - 0104: "Fort law"
 - 0105: "Schelchberg"
 - 0106: "New Guernsey"
 - 0107: "No Law"
 - 0108: "Brix"
 - 0109: "Island view"
 - 0110: "Law Island"
 - 0111: "Estoria"
 - 0112: "Kings"
 - 0113: "Ardler"
 - 0114: "Brocklyn"
 - 0115: "Tellburg"
 - 0116: "Brocklyn docks"
 - 0117: "Island city"
 - 0118: "Island heights"
 - 0119: "Park"
 - 0120: "Nixon Island"
 - 0121: "North"
 - 0122: "East"
 - 0123: "South"
 - 0124: "West"
 - 0125: "Central"
 - 0126: jingle
 - 0127: jingle
 - 0128: nothing
 - 0129: nothing
 - 0130: jingle

## LEVEL002

All sounds are identical for LEVEL001 and LEVEL002 (same md5sum of the
extracted WAV file) except for 100-120 (district names) and 126, 127, 129 and
130.

 - 0100: "Telephone Hill"
 - 0101: "Wood Hill"
 - 0102: "Soviet Hill"
 - 0103: "Sunrise"
 - 0104: "Richman"
 - 0105: "Atlantic Heights"
 - 0106: "Aye Valley"
 - 0107: "Chinatown"
 - 0108: "Eagleside"
 - 0109: "Woodside"
 - 0110: "Potato"
 - 0111: "Excalibur"
 - 0112: "Glen Wood"
 - 0113: "Market"
 - 0114: "Marina"
 - 0115: "Sailors Wharf"
 - 0116: "Sunview"
 - 0117: nothing
 - 0118: nothing
 - 0119: nothing
 - 0120: nothing

 - 0126: jingle
 - 0127: jingle
 - 0128: nothing
 - 0129: jingle
 - 0130: nothing
 
## LEVEL003

All sounds are identical for LEVEL001 and LEVEL003 (same md5sum of the
extracted WAV file) except for 100-120 (district names) and 126, 128, 129 and
130.

 - 0100: "Vice Beach"
 - 0101: "Banana Grove"
 - 0102: "Felicity"
 - 0103: "Richman Heights"
 - 0104: "Little Bogota"
 - 0105: "Greek Heights"
 - 0106: "Little Dominica"
 - 0107: "Coral City"
 - 0108: "Miramire"
 - 0109: "Vice Shores"
 - 0110: nothing
 - 0111: nothing
 - 0112: nothing
 - 0113: nothing
 - 0114: nothing
 - 0115: nothing
 - 0116: nothing
 - 0117: nothing
 - 0118: nothing
 - 0119: nothing
 - 0120: nothing

 - 0126: jingle
 - 0127: jingle
 - 0128: jingle
 - 0129: jingle
 - 0130: nothing
