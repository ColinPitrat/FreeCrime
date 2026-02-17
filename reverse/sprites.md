This file lists some interesting sprites identified.

## NYC - STYLE001.G24

TODO: Understand these 13 sprites properly:

Starting at `SPR_OBJECT` (83), some objects have 13 sprites in total:
 - 1 at rest
 - 8 for the animation when bumped into by a car
 - 4 for the animation of sinking into water

TODO: These sprite numbers vary for a different style file. Find the proper 
object number associated and verify that it is fixed across all style files.

 - 261-264: Tank missile / Rocket launcher "bullet"
 - 265: Look like a bunch of rocks. Tank turret very damaged/explosed? (TODO: confirm)
 - 266: Tank turret
 - 267: Tank cannon
 - 268: Tank turret damaged
 - 269: Tank cannon damaged? (TODO: confirm)

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
