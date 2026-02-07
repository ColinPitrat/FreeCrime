# GTA 1 DAT file format 

The `PLAYER_A.DAT` file is the save file and contains scores, players, levels
completed, etc ...

This file is incompatible between different versions of the game.

The format described here is the one from the original GTA 1 game.
For variants (London 1969, London 1961, GTA1 Classics), see
https://misc.daniel-marschall.de/spiele/gta1/player_a_format.txt

The overall format is:

| name | size | notes |
|------|------|-------|
| sound volume | uint8 | 0=off, 1..7 as in Options menu |
| music volume | uint8 | 0=off, 1..7 as in Options menu |
| text speed | uint8  | 1..3 - slow, normal, fast |
| radio mode | uint8  | 0=radio, 1=constant |
| resolution | uint8  | 0=640x480x16, 1=800x600x16, 2=1024x768x32, 3=640x480x32, 4=800x600x32, 5=1024x768x32 |
| default resolution | uint8  |  |
| reserved | uint8  |  |
| transparency effects | uint8  |  0=off, 1=on |
| deathmatch mode | uint8  |  0=score, 1=kills |
| deathmatch score | uint32  |  |
| deathmatch kills | uint32  |  |
| language | uint8  | 0=english, 1=french, 2=german, 3=italian, 4=japanes, 99=uncensored english (SPECIAL.FXT) |
| highscores | HighScores[18] | |
| players | Player[8] |
| selected player ID | uint8 | 1..8 |
| multiplayer host name | char[15] | default to "GTA Game" |
```

The high scores are stored in the HighScores structure:

| name | size | notes |
|------|------|-------|
| score | int32 | -1 if no highscore |
| player name | char[15] | |

There are 3 high scores per level.

The players information is stored in the Player structure:

| name | size | notes |
|------|------|-------|
| name | char[15] | |
| highscores | int32[6] | One high score per level |
| reserved | int32 | |
| multiplayer selected area | int32 | 0=Liberty City, 1=San Andreas, 2=Vice City |
| reserved | int32 | |
| multiplayer selected game | int32 | 0=deathmatch, 1..3=canonball 1 to 3 |
| video unlocked | uint32[6] | 0=locked, 1=unlocked |
