# GTA 1 MISSION.INI file format

Project Cerbera has some nice (though incomplete) explanations on the way the MISSION.INI file works:
 - https://projectcerbera.com/gta/1/tutorials/smallest
 - https://projectcerbera.com/gta/1/tutorials/insane-stunts
 - https://projectcerbera.com/gta/tutorials/

## Common rules

### Coordinates system

Coordinates in the world are represented as a triplet (x, y, z).

The values can either be:
 - in blocks, between 0 and 255 for x and y, and between 0 and 5 for z (a z of 0 will not be visible)
 - in pixels (64 per block), between 0 and 16320 for x and y, and between 0 and 320 (maximum value observed) for z

One block is the width of a single lane.
x increases going east and y increases going south.

Orientation/rotation for object can often be specified with a number between 0 and 1023 (going from 0 to 360°, so 1 unit = 0.351°).
For block related things, it can be specified with a number between 0 and 3 but the meaning varies (see PARK and BARRIER for two examples with different usages).

### Comments

It is possible to add comments in the file using curly braces. The comments can be multi-lines.

Example:
```
{ This sets the player starting position and orientation. }
294 1 (105,119,4) PLAYER 293 256
```

### Jumping to commands

Many commands are checking something and allow to jump to two different other commands depending on the result.

For example:
```
10000 SURVIVE 0 10050 10100 0 0
```
Tests whether the player survives and would jump to command 10050 if they do or to command 10100 if they don't.

The second and third parameters can be:
 - -1, in which case the routine is stopped
 - 0, in which case the command jumped to is the next instruction
 - a positive number, which is the command that will be jumped to

### Monetary reward

The last parameter of a command is often (always?) a monetary reward.

For example:
```
10000 SURVIVE 0 -1 -1 0 5000
```
Will reward the player with 5000$ if they survive.

## Structure

### Objects
Example:
```
0 1 (108,108,4) TELEPHONE 0 768
```

The second value (the `1`) means the object is permanent. If absent, the object will be cleaned-up by the RESET command.

TODO: Find more details about it.
I think RESET only cleans objects created by the mission itself, and therefore in some cases the 1 being present or not doesn't matter.

For example:
```
156 1 (0,0,0) MISSION_COUNTER 4 0
(...)
9000 (0,0,0) MISSION_COUNTER 16 0
```

Both mission counters should be permanent and not be cleaned by a RESET command.

### Commands
Example:
```
9 SURVIVE 0 0 0 5 0
10 ARROW 137 0 0 0 0
11 MOBILE_BRIEF 0 0 0 0 1001
12 STARTUP 303 30000 30000 0 0
```

### Startup commands

## All objects

### `BARRIER`
Places a barrier.

Example:
```
154 1 (98,49,3) BARRIER 2 7 0
```
Creates a barrier with the object ID 154 at coordinates (98, 49, 3).
The first parameter after `BARRIER` specifies the side of the block on which it will stand (0=west, 1=east, 2=north, 3=south)
The second parameter (7 in the example) doesn't seem to have any effect. In the game's files, it's always either 4 or 7.
TODO: Apparently it is the vehicle model or type that is allowed. Test and update.
The third parameter is always 0 in the game's files. It determines the sprite used for the barrier.

Some nice documentation at https://projectcerbera.com/gta/1/tutorials/park

### `BASIC_BARRIER`
Places a barrier. No noticeable difference with `BARRIER` but it's used in 4 places in the original game's MISSION.INI.

### `BLOCK_INFO`
Creates an ID to reference a block in the map.

This reference can be used later with `CHANGE_BLOCK` or `CHANGE_TYPE` to update the map content.

Example:
```
279 1 (151,44,2) BLOCK_INFO 0 0
```
Allows to reference the block at (151,44,2) with the object ID 279.

The first parameter is:
 - the new value for the block when used with `CHANGE_BLOCK`
 - the face to change when used with `CHANGE_TYPE` (0=west, 1=east, 2=south, 3=north, 4=lid)
The second parameter is:
 - not used with `CHANGE_BLOCK`?
 - the new face value with `CHANGE_TYPE`

### `BOMBSHOP`
Place a bombshop at the provided position.

Note that the bombshop is invisible. It will automatically place a bomb in the
vehicle when going on the tile with one, provided the player has enough money.

Example:
```
176 1 (1,156,3) BOMBSHOP 0 0
```
Creates a bombshop with the object ID 176 at coordinates (1, 156, 3).

### `BOMBSHOP_COST`
Set the cost of placing a bomb in a vehicle in all bombshops.

Example:
```
278 1 (0,0,0) BOMBSHOP_COST 1000 0
```
Sets the cost of placing a bomb in a car to 1000 (instead of the default 5000).

### `CARBOMB_TRIG`
From the name, this should trigger the bomb of a car but so far, couldn't make it work.
TODO: Reseach more.

### `CARDESTROY_TRIG`
### `CARSTUCK_TRIG`
### `CARTRIGGER`
A trigger on entering a particular car.

Example:
```
296 1 (147,73,4) CARTRIGGER 2500 295
```
When entering the car object 295, jumps to command 2500.

The first parameter is the command to jump to when the player enters the car.
The second parameter is the ID of the car object subject to the trigger.
The position of the trigger doesn't really matter. In game's files, it is set either to the same position as the car or to (0,0,0).

### `CARWAIT_TRIG`
### `CHOPPER_ENDPOINT`
### `CORRECT_CAR_TRIG`
### `COUNTER`
### `CRANE`
### `DOOR`
TODO
Some nice documentation at https://projectcerbera.com/gta/1/tutorials/park
### `DUM_MISSION_TRIG`
### `DUMMY`
### `FUTURE`
Define an emplacement where a future object can be created with MAKEOBJ.

Example:
```
240 (6720,2624,256) FUTURE 47 0
```
Defines a drop-off point (47) as the emplacement 240 at coordinates (6720,2624,256) (in pixels, with 64 pixels per block, so in block (105,41,4)).

The first parameter is the type of object. See `object.txt` at https://projectcerbera.com/gta/1/tutorials/reference for a list of objects. (TODO: copy the file here with proper attribution for posterity?)
The second parameter should logically be an orientation (rotation) of the object, between 0 and 1023, but it actually doesn't seem to have any effect.

### `FUTURECAR`
### `FUTUREPED`
### `GTA_DEMAND`
### `GUN_SCREEN_TRIG`
### `GUN_TRIG`
### `MISSION_COUNTER`
### `MISSION_TOTAL`
### `MODEL_BARRIER`
### `MOVING_TRIG`
### `MPHONES`
### `PARKED`
Creates  a parked car.

First parameter is the car model (TODO: Create car_models.md describing car models depending on the level (if it does depend on the level), maybe including London extension pack).
Second parameter is the angle of the car.

### `PARKED_PIXELS`
### `PED`
### `PEDCAR_TRIG`
### `PHONE_TOGG`
### `PLAYER`
Position the player at its start position. For a multiplayer mission, there can be multiple player positioned.

Example:
```
293 1 (106,119,4) PARKED 31 0
294 1 (105,119,4) PLAYER 293 256
```
Here, object 294 is the player. It is positioned at (105,119,4).
The first parameter, 293, refers a `PARKED` object before. This parked object is a car that the player owns (entering it doesn't count as stealing).
The second parameter is the orientation of the player. It goes from 0 to 1023. 0 means pointing south, 256 is east, etc....

### `POWERUP`
Creates a powerup (weapon, life, multiplier, ...).

Example:
```
271 (70,85,4) POWERUP 2 500
```
Creates object 271 at the position (70,85,4). This is of type 2 (machine gun)
and this is a kill frenzy that lasts 15 seconds (500 frames). This particular
kill frenzy requires 1000 points, but unsure how the number of points is
configured (if it is).

The first parameter is the power-up type (see [powerups.md](powerups.md) for the
full list). 1=Pistol, 2=Machine gun, 3=Rocket launcher, 4=Flame thrower,
6=Speedup, 9=Cop bribe, 10=Armour, 11=Multiplier, 12=Get out of jail, 13=Life

The second parameter is the number of ammunitions (for weapons). If 0, the
default amount is provided.

TODO: Understand how the number of points required by the kill frenzy is
configured.

TODO: Verify if the amount can be changed for non-weapons: speedup (duration?),
armour (more hits?), multiplier (more than 1?), get out of jail (unlikely),
extra life (more than 1?)


### `SECRET_MISSION_COUNTER`
### `SPECIFIC_BARR`
### `SPECIFIC_DOOR`
### `SPRAY`
Creates a spray shop for a given color.

Example:
```
160 1 (44,139,3) SPRAY 1 0
```

The color can range from 1 to 6.
The second parameter doesn't seem to be used and is always 0 in the game's resources.

### `TARGET`
### `TARGET_SCORE`
### `TELEPHONE`
Creates a public phone.

Example:
```
0 1 (108,108,4) TELEPHONE 0 768
```
Which creates a telephone with ID `0` at position (108, 108, 4) pointing toward south (768).
The first parameter doesn't seem to be used and is always 0 in the game's resources.
The second value is always 1 and the fifth value is always 0. Changing these values doesn't seem to have any effect.

### `TRIGGER`
An invisible objects that triggers an action when the player reaches its position.

Example:
```
136 (60,246,4) TRIGGER 2635 0
```
Creates a trigger with id `136` at position `(60,246,4)` which triggers command 2635 when reached.
The second parameter is the range of the trigger ([Moore neighborhood](https://en.wikipedia.org/wiki/Moore_neighborhood)). A range of 0 checks only the current block.

## All keywords

### `ADD_A_LIFE`
Give an additional life to the player.

Example:
```
11022 ADD_A_LIFE 0 0 -1 -1 0
```

In the game's files, the parameters are always `0 0 -1 -1 0`.

### `ANSWER`
Check whether the player answers a phone in time.

Example:
```
4040 ANSWER 112 0 4270 10 2500
```

The player must answer the phone 112 in less than 20 seconds. If they succeed,
the command at 4270 will be executed.

The first parameter is the object ID of the phone to answer.
The second parameter is the command to execute when the phone is NOT answered.
The third parameter is the command to execute when the phone is answered.
The fourth parameter is the duration given to the player. The scale doesn't seem linear. 10 is roughly 20 seconds, 20 is roughly 1 minute, 30 is roughly 2 minutes.
The fifth parameter doesn't seem to have any effect. It seems (from the values in the file) that the intent was a monetary reward but if so, this doesn't seem to work!

TODO: Clarify the timing (test more extreme values).

### `ARMEDMESS`
Display a message that a bomb was armed.

Example:
```
5050 ARMEDMESS 0 0 0 0 0
```

This is always used with 0 for the 5 parameters in the game's files.

The fifth parameter can provide a monetary reward.

### `ARROW`
Makes the arrow point to an object.

Example:
```
10 ARROW 137 0 0 0 0
```

Makes the arrow point towards the object `137`.
The next two parameters seem to be an instruction to loop to.

For example:
```
430 IS_PED_IN_CAR 294 430 0 -1 0
432 SURVIVE 0 0 0 25 0
433 MOBILE_BRIEF 0 0 0 0 1016
434 ARROWCAR 189 0 0 0 0
436 STEAL 189 0 -1 0 0
437 SURVIVE 0 0 0 20 0
438 ARROW 191 430 430 0 0
```

When executing 438, the arrow will point towards object `191` and next instruction will be 430.
The instruction 430 loops to itself unless the player enters the car `294`. Once the player does so, they are directed to stealing the car `189`.
This seems to be a way to keep the process running? Not 100% sure...

### `ARROWCAR`
### `ARROW_OFF`
Hide the arrow

### `ARROWPED`
### `BANK_ALARM_OFF`
### `BANK_ALARM_ON`
### `BANK_ROBBERY`
### `BRIEF`
### `CANCEL_BRIEFING`
### `CAR_ON`
### `CHANGE_BLOCK`
Modify a block in the map.

The block modification must have been prepared with `BLOCK_INFO`.

Example:
```
5720 CHANGE_BLOCK 229 0 0 0 0
```
Executes the block change configured in object 229.

The first parameter is the `BLOCK_INFO` object ID to execute.
The second parameter is the next instruction to execute (always 0 in the game's files).
The third parameter doesn't have any actual effect? (always 0 in the game's files).
The fourth parameter doesn't have any effect? (always 0 in the game's files);
The fifth parameter is a monetary reward? (TODO: test - always 0 in the game's files);

### `CHANGE_PED_TYPE`
### `CHANGE_TYPE`
Modify the face of a block in the map.

The block modification must have been prepared with `BLOCK_INFO`.

Example:
```
5710 CHANGE_TYPE 223 0 0 0 0
```
Executes the block change configured in object 223.

The first parameter is the `BLOCK_INFO` object ID to execute.
The second parameter is the next instruction to execute.
The third parameter doesn't have any actual effect? (sometimes -1 in the game's files).
The fourth parameter doesn't have any effect? (always 0 in the game's files);
The fifth parameter is a monetary reward? (TODO: test - always 0 in the game's files);

### `CHECK_CAR`
### `CLOSE_DOOR`
### `COMPARE`
### `CRANE`
### `DEAD_ARRESTED`
### `DECCOUNT`
### `DESTROY`
### `DISABLE`
### `DISARMMESS`
### `DO_GTA`
### `DO_MODEL`
### `DONOWT`
### `DOOR_OFF`
### `DOOR_ON`
### `DO_REPO`
### `DROP_ON`
### `DROP_WANTED_LEVEL`
### `DUMMY_DRIVE_ON`
### `DUMMYON`
### `ENABLE`
### `END`
End the game.

Example:
```
8180 END 1 0 0 0 0
```
Ends the game, winning.

First parameters tells what type of end this is:
 - 0, 7: quit, incomplete (m22incomplete)
 - 1: win (m22success) - plays the sequence
 - 2: failed (m22failed)
 - 3: dead (m22dead)
 - 4: arrested (m22arrest)
 - 5: timeout (m22timeout)
 - 6: timeover (m22timeover) - multiplayer game
 - 8: win multiplayer (m22score) - multiplayer game
 - 10: win race (m22cannon) - multiplayer game
 - 11: end of demo (m22demo)

Other parameters don't seem to have any effect.

### `EXPL_NO_FIRE`
### `EXPLODE`
### `EXPLODE_CAR`
### `EXPL_LAST`
This keyword is only used in UK mission file (GTA London mission packs).
### `EXPL_PED`
### `FREEUP_CAR`
### `FREEZE_ENTER`
Prevent the player to exit the vehicle they are in.

Example:
```
3516 FREEZE_ENTER 0 0 0 0 0
```
All parameters are always 0 in the game's files.

### `FREEZE_TIMED`
### `FRENZY_BRIEF`
Display a briefing message for a frenzy.

Example:
```
9010 FRENZY_BRIEF 0 0 0 16 1426
```

Displays the message 1426 (which usually contains the number of points and
sometimes the duration), independently of the actual parameters of the frenzy.

The first parameter is always 0 in the game's files.
The second parameter is a command to jump to next. If 0, it is simply the next
command in the script.
The third parameter is either 0 or -1 in the game's files but its effect is
unclear.
The fourth parameter is either 0, 16 or 24 in the game's files but its effect is
unclear.

### `FRENZY_CHECK`
Verifies that a frenzy challenge is achieved in a given amount of time.

Example:
```
9030 FRENZY_CHECK 1000 0 9030 0 20000
```
Checks that the player makes 1000 points (first parameter) in the amount of time provided.
If the frenzy goal is not reached, it jumps to the 3rd parameter (so here it loops to 9030).
If the frenzy goal is reached:
 - it jumps to the 2nd parameter (here 0, it doesn't jump anywhere)
 - the player gets the number of points provided in the 5th parameter (here 20000).

The 4th parameter is always 0 in the game's file.

TODO: Understand where the duration of the frenzy comes from. There doesn't seem
to be any parameter for it, so it may be in the game engine? But there are
various timings available from the messages: 20 seconds, 80 seconds, 100 seconds,
2 minutes, ...

### `FRENZY_SET`
### `GENERAL_ONSCREEN`
### `GET_CAR_INFO`
### `GET_DRIVER_INFO`
### `GOTO`
Test if the player got to a point.

Example:
```
29020 GOTO 10010 0 29041 100 0
```
Checks if players got to trigger 10010 in 100 frames (4 seconds). If not, jumps to command 29041, otherwise proceed with the next command.

The first parameter is the target object to reach.
The second parameter is the command to jump to if reached.
The third parameter is the command to jump to if not reached in time.
The fourth parameter is the time to reach the target.
The fifth parameter may be a monetary reward for reaching it (TODO: check that).

### `GOTO_DROPOFF`
### `HELL_ON`
### `HUNTOFF`
### `HUNTON`
### `INCCOUNT`
### `INC_HEADS`
### `IS_GOAL_DEAD`
Test if an object is dead.

Example:
```
29030 IS_GOAL_DEAD 85 29041 0 0 0
```
Checks to see if object 85 (the PLAYER in this case) is dead. If so, it goes to command 29041. If not, it proceed to the next command.

The first parameter is the object to check for death.
The second parameter is the command to jump to if the object is dead.
The third parameter is the command to jump to if the object is NOT dead.
The fourth parameter may be a duration for which to check? (TODO: verify this)
The fifth parameter may be a monetary reward for the target being dead? (TODO: check that).

### `IS_PED_ARRESTED`
### `IS_PED_IN_CAR`
### `IS_PED_STUNNED`
### `IS_PLAYER_ON_TRAIN`
### `IS_POWERUP_DONE`
### `KEEP_THIS_PROC`
### `KF_BRIEF_GENERAL`
### `KF_BRIEF_TIMED`
### `KF_CANCEL_BRIEFING`
### `KF_CANCEL_GENERAL`
### `KF_PROCESS`
### `KICKSTART`
### `KILL_CAR`
### `KILL_DROP`
### `KILL_OBJ`
Destroy an object previously created with MAKEOBJ.

Example:
```
816 KILL_OBJ 140 0 -1 0 0
```

The first parameter is the object ID to kill.
The second parameter is the command to jump to next (0 to continue, -1 to stop).
The third parameter doesn't seem to have any effect (although it is sometimes set to -1 in the game's files).
The fourth parameter doesn't seem to have any effect and is always 0 in the game's files.
The fifth parameter is a monetary reward.

### `KILL_PED`
### `KILL_PROCESS`
### `KILL_SIDE_PROC`
### `KILL_SPEC_PROC`
### `LOCATE`
### `LOCK_DOOR`
### `MAKEOBJ`
Create an object previously specified by FUTURE.

Example:
```
760 MAKEOBJ 53 0 0 0 0
```
Creates the object 53.

The first parameter is the object ID to create.
The second parameter is a command to jump to (if 0, continues with the next command).

### `MESSAGE_BRIEF`

Displays a message. For example:
```
29038 MESSAGE_BRIEF 0 0 0 0 2514
```
displays the "INSANE STUNT BONUS" (2514) message in big in the middle of the screen.

The first four parameters role is less obvious.

The first and fourth parameters are always 0.

The second parameter can be 0 or -1. When not, it seems to be an action number which can be executed in parallel of the message being displayed. For example:
```
32441 MESSAGE_BRIEF 0 32284 0 0 2503
```

Displays the message `KILL FRENZY!` (2503) but it seems to refer to action 32284 which is:
```
32284 DECCOUNT 23330 32213 32213 0 0
```
and 23330 is:
```
23330 (0,0,0) SECRET_MISSION_COUNTER 17 0
```
Where 17 is the number of secret missions for this level.
and 32213 is:
```
32213 FRENZY_BRIEF 0 0 0 0 3534
```
with the message `YOU GOT 100 SECONDS TO DO $70,000 OF DAMAGE! MOVE IT OUT!` (3534) being referenced.

The third parameter is either -1 or 0 except for a single occurrence:
```
4289 MESSAGE_BRIEF 0 4315 4315 0 2500
```

The message 2500 is `MISSION COMPLETE!`. The action 4315 is:
```
4315 SURVIVE 0 0 0 15 0
```

It seems that repeating it as the third parameter means the execution should continue there instead of on the next action.

The meaning of -1 is unclear.

### `MISSION_END`
### `MOBILE_BRIEF`
### `MODEL_HUNT`
### `MPHONE`
### `NEXT_KICK`
### `OBTAIN`
### `OPEN_DOOR`
### `PARK`
Checks if a given car is parked in a garage.

Example:
```
436 PARK 178 0 24500 3 35000
```
Detects when the player is in the DOOR 178. If so, automatically respawn the player outside, remove the car and close the door.
24500 is a DUMMY at the same coordinates as the DOOR
3 tells to respawn the player east of the door (0=south, 1=west, 2=north, 3=east)
35000 is the monetary reward

Some nice documentation at https://projectcerbera.com/gta/1/tutorials/park

### `PARKED_ON`
### `PARKED_PIXELS_ON`
### `P_BRIEF`
### `P_BRIEF_TIMED`
### `PED_BACK`
### `PED_ON`
### `PED_OUT_OF_CAR`
### `PED_POLICE`
### `PED_SENDTO`
### `PED_WEAPON`
### `PIXEL_CAR_ON`
### `PLAIN_EXPL_BUILDING`
### `PLAYER_ARE_BOTH_ONSCREEN`
### `POWERUP_OFF`
### `POWERUP_ON`
### `RED_ARROW`
### `RED_ARROW_OFF`
### `RED_ARROWCAR`
This keyword is only used in UK mission file (GTA London mission packs).
### `REMAP_CAR`
This keyword is only used in UK mission file (GTA London mission packs).
### `REMAP_PED`
### `RESET`
Cleanup objects after a mission ends.

The ending of every mission should have a RESET command which automatically erases every mission object from the game environment unless it has 1 placed after the object number and before the coordinates.

### `RESET_KF`
### `RESET_WITH_BRIEFS`
### `RETURN_CONTROL`
### `SCORE_CHECK`
### `SENDTO`
### `SETBOMB`
Configures a bomb in a vehicle.

Example:
```
3540 SETBOMB 297 0 0 6 0
```

Configures a bomb in the vehicle 297.

The first parameter is the vehicle ID on which to set a bomb.
The fourth parameter tells when the bomb should explode:
 - 0: the bomb is disarmed
 - 1: ??
 - 2: the explosion triggers after a few seconds
 - 4: if the vehicle bumps a bit too much, the bomb explodes
 - 6: if the vehicle speed falls below 50mph
The fifth parameter is a monetary reward immediately granted to the player.

TODO: understand the roles of the second, third and fifth parameters

### `SET_KILLTRIG`
### `SET_NO_COLLIDE`
### `SET_PED_SPEED`
### `SETUP_REPO`
### `SPEECH_BRIEF`
### `START_MODEL`
### `STARTUP`
### `STEAL`
### `STOP_FRENZY`
### `SURVIVE`
Wait to see if the player survive for a given amount of time.

Example:
```
3560 SURVIVE 0 0 0 250 10000
```
Checks that the player survives for 250 frames (10 seconds). If the player
survives, they are rewarded with 10000$.

TODO: Understand the role of the first parameter
The second parameter can hold a command to jump to if the player survives.
The third parameter can hold a command to jump to if the player doesn't survive.
The fourth parameter holds the duration for which the player must survive (in frames, i.e. in 25th of a second).
The fifth parameter holds a monetary reward for surviving.

### `THROW`
### `UNFREEZE_ENTER`
Allow the player to exit the vehicle they are in.

Example:
```
3580 UNFREEZE_ENTER 0 0 0 0 0
```

All parameters are always 0 in the game's files except 2nd and 3rd which can
probably hold a command to jump to.

### `UNLOCK_DOOR`
### `WAIT_FOR_PED`
### `WAIT_FOR_PLAYERS`
### `WRECK_A_TRAIN`
