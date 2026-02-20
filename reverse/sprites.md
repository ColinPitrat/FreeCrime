This file lists some interesting sprites identified.

## NYC - STYLE001.G24

TODO: Understand these 13 sprites properly:

Starting at `SPR_OBJECT` (83), some objects have 13 sprites in total:
 - 1 at rest
 - 8 for the animation when bumped into by a car
 - 4 for the animation of sinking into water

TODO: These sprite numbers vary for a different style file. Find the proper 
object number associated and verify that it is fixed across all style files.

There are 2 sets of sprites for the same UI elements, the first set being with
lower resolution. The second set has one more sprite because the area name is in
one sprite for the low-resolution set whereas it's in two sprites in the
high-resolution set.

In what follows, sprite numbers for these elements are of the form:
`<low> & <high>`.

UI elements:
 - 0 & 24: Yellow arrow
 - 1 & 25-26: Area name box
 - 2 & 27: Car name box
 - 3 & 28: Pager
 - 4 & 29: Blinking light of the pager (lit on)

Selected weapon icons:
 - 5 & 30: Pistol
 - 6 & 31: Machine gun
 - 7 & 32: Rocket launcher icon
 - 8 & 33: Flame thrower
 - 9 & 34: Petrol bomb icon

Briefs?
 - 10 & 35: Mouth icon (displayed on the left of a spoken message, for example when reselling a car at the docks)
 - 11 & 36: Payphone icon (displayed on the left of payphone brief text)
 - 12 & 37: Mobile phone icon (displayed on the left of mobile brief text)
 - 13 & 38: Cop icon (displayed left of criminal record when getting out of jail)
 - 14 & 39: Information icon (displayed left of the information when taking an information bonus)
 - 15 & 40: Kill frenzy icon (displayed left of the kill frenzy instructions)
 - 16-17 & 41-42: Policeman head (when police researches the player)

Active bonuses icon:
 - 18-19 & 43-44: Armor
 - 20-21 & 45-46: Get out of jail free
 - 22-23 & 47-48: Speed bonus

 - 224-229: Object #15: Puddle of blood disappearing (or appearing if played in reverse)
 - 229: Object #77: Spray of blood (e.g. when pedestrian shot while running, there will be a few spray of blood before the spot where they fall and where the puddle will appear)
 - 437-448: Object #63: Another puddle of blood?
 - 449: Object #64: Another puddle of blood?

 - 261-264: Tank missile / Rocket launcher "bullet"
 - 265: Look like a bunch of rocks. Tank turret very damaged/explosed? (TODO: confirm)
 - 266: Tank turret
 - 267: Tank cannon
 - 268: Tank turret damaged
 - 269: Tank cannon damaged? (TODO: confirm)

 - 275-276: Ringing phone
 - 279-290: Suitcase falling into the water

 - 391-396 (maybe 397 too?): Doors of the train opening/closing

 - 475: Bullet (Pistol & Machine gun)
 - 476-486: Flame thrower "bullet"

Starting at `SPR_PED` (645):

- 645-652: Player walking backward
- 653-660: Player running (going forward)
- 661: Player driving
- 662-669: Player exiting vehicle?
- 670-677: Player entering vehicle? (but looks the same as exiting, just backward)
- 678: Player driving (again?)
- 734-735: Player shooting with a gun
- 744-751: Player shooting with a gun
- 752-759: Player walking and shooting with a gun

Vehicles of type motorcycle have 13 sprites:

- 953-965: bike
- 966-978: superbike

Traffic lights:
- 942: red
- 943: green
- 944: amber
- 945: off
- 946: ???
- 947: ??? (look like a traffic light seen from top, light not visible, but couldn't find a place where it would be used)

- 986-1033: 4 sets of 12 sprites for the explosions for the four directions (in order: NW, NE, SW and SE)

## remaps

The following is true for all levels.

In 8 bits, remaps for pedestrians seem to go from 125 to 188 although there's a
doubt for the lower boundary (but starting at 125 aligns the numbers well with
the 24 bits remaps).
In 24 bits, the remaps for pedestrians are always the last 64 in the `newcarclut`
section.
Playable characters have the following remaps (indexed from 0):

| name | remap 8 bits | remap 24 bits |
|------|--------------|---------------|
| Ulrika | 149 | 24 (-40) |
| Travis |   0 |        0 |
| Katie  | 153 | 28 (-36) |
| Mikki  | 157 | 32 (-32) |
| Divine | 161 | 36 (-28) |
| Bubba  | 165 | 40 (-24) |
| Troy   | 169 | 44 (-20) |
| Kivlov | 173 | 48 (-16) |

It seems that the sequence of 8 bits remap is stored starting at offset 0x1C8EE4
in GTA8.EXE with a value every 4 byte, in the same order as characters names
appear later in the file (at 0x1D361A).

Hare Krishna is the remap 18 in 24 bits and 143 in 8 bits.

Paramedic use the same sprite as policemen but with a remap:
 - in 24 bits: remap 23 (or -41 starting from the end of newcarclut).
 - in 8 bits: remap 148
