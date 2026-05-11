#!/bin/bash

# This script was used to investigate the mystery of London water.

CITY="London"
CITY="Liberty City"

# Decide the style file and the foreground color based on the city.
case $CITY in
  "London")
    #style_file=path-to-gtadata/UK/Style001.g24
    fg=0
    ;;
  "Liberty City")
    #style_file=path-to-gtadata/STYLE001.G24
    fg=200
    ;;
  *)
    echo "Unsupported city '$CITY'"
    exit 1
    ;;
esac

# The following shows that anim 0 (water) has frame_count=11. So the animation
# should have a total of 12 frames: the original lid block (41) and all the aux
# blocks listed (25-35). 
python3 reverse/modify_gry.py --print header --print anim.num_anims --print anim.anims[0] --print anim.anims[1] --print anim.anims[2] --print anim.anims[3] --print anim.anims[4] ${style_file}

# However, when displaying it (reverse/display_gry.py or `cargo run -- display`)
# there's a gray frame in the middle of the animation.
# Checking the exported bitmaps (`reverse/modify_gry.py export` or
# `cargo run -- extract`) confirms that aux_block_035 is a noisy gray block, not
# water.

# Modifying the style to see what the original game displays:
python3 reverse/modify_gry.py ${style_file} --update_block lid,41,0,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,25,1,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,26,2,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,27,3,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,28,4,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,29,5,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,30,6,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,31,7,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,32,8,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,33,9,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,34,A,${fg},1
python3 reverse/modify_gry.py ${style_file} --update_block aux,35,B,${fg},1

# Running the original game, in London, we see that the animation loops from 0
# to A. The tile B is never displayed.
# However, in Liberty City all 12 tiles are displayed.

# Ultimate test:
# 1) Copy path-to-gtadata/UK/Style001.g24 to path-to-gtadata/STYLE001.G24
#    The water does display the gray tile.
# 2) Copy path-to-gtadata/UK/UK.CMP to path-to-gtadata/NYC.CMP
#    This doesn't work!
# 3) Copy path-to-gtadata/UK/MISS.INI to path-to-gtadata/MISSION.INI
#    Edit it to modify the paths to point to NYC.CMP
#    The water does display the gray tile.
# So the information that only 11 tiles should be displayed, not 12, for this
# particular animation, is not in the style file, not in the map and not in the
# mission.ini.

# This suggests this is hard-coded in the game engine!
# This is totally bonkers. It seems that instead of fixing the one byte
# framecount of the animation, the developers made an exception for this
# particular animation in the game engine. It's really hard to believe.
