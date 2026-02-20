## Entering cars

When pressing Enter, the player enter up to 3 blocks away.
Assuming numbers are blocks, 0 is where the player is standing, it will be able
to enter a car which is in any of the tile with a number strictly lower than 3:
```
3 3 3 3 3 3 3
3 2 2 2 2 2 3
3 2 1 1 1 2 3
3 2 1 0 1 2 3
3 2 1 1 1 2 3
3 2 2 2 2 2 3
3 3 3 3 3 3 3
```

If there are multiple cars in the area, the closest one (in pixels from center)
is picked.

## Driving cars

The cars don't turn when stopped (duh!). They have a fixed turning radius,
meaning that they turn on a fixed circle independent of their speed.
Concretely, this means that the car is rotating at an angular speed which is
proportional to its speed.

The turning radius is different (usually much smaller) when going backward.

In practice, the actual turning radius is slightly bigger when starting to turn
from high speed because the car is drifting a bit at the beginning. This is
usually quite small though.

The turning radius gets much smaller when using the handbrake, together with a
large reduction of the speed.

## Reappearing bonuses

Some weapons (e.g. Pistol, Machine Gun) respawn after they are taken, after a
variable amount of time. This is configured in MISSION.INI by having a TRIGGER
on the same coordinates that checks if the power up was taken
(`IS_POWERUP_DONE`), waits for the player to survive a given time (SURVIVE) and
then re-enable the powerup.

## Traffic lights

Traffic lights are drawn without taking the perspective into account. They are
just a sprite displayed high above the middle of the street. The side showing
the light is always visible even when it doesn't make sense from a perspective
point of view.

Traffic lights go through a sequence: green - amber - red - blinking amber

When a traffic light is not green, cars stop on the tile which is before it
(where before means on the side of the traffic light where the light is).
Interestingly, the cars stop no matter the direction in which they are going.
So some cars will stop after exiting the junction if the traffic light for the
other direction is not green (typically cars turning on a T junction).

## Area names

Navigation data from the CMP file contains a list of areas defined as rectangles
on the map. For example, "Nixon Island", "Hackenslash", etc...

These areas are subdivided in 9 rectangles, presumably splitting both the
rectangle width and height in 3 equal parts.

Both the sound files (SDT/RAW) and the text files (FXT) contain prefixes for the
subdivisions (North, South, East, West, Central, Southeast, Southwest, Northeast
and Northwest). In text files, these are strings with ids `n`, `s`, `e`, `w`,
`c`, `se,` `sw`, `ne` and `nw`. In sound files, there are only 5 sounds for
"North", "East", "South", "West" and "Central" which can be composed as needed.

The area names have a sound number defined in the navigation data in the CMP
file: the `sample` field. It's an offset in the list of files where the first
sample (offset 1) is the 100th sound. The exception is sample 0 which
corresponds to the 120th sound.

In text files, the string with the name of the area is `00xarea0yy` where x is
the city (1=Liberty City/NYC, 2=San Andreas/SANB, 3=Vice City/MIAMI).

For example, the first three areas in Liberty City are:
```
[001area000]Nixon Island
[001area001]Liberty City
[001area002]Hackenslash
```

To be noted that areas can overlap though in practice, the only case where it
happens is for Liberty City and Vice City for which there are areas defined for
the whole city. It's not clear what these areas are used for nor why the same is
not done for San Andreas.
