# GTA 1 GRY file format

The GRY file format stores graphics and objects data.

Most of the information in this page comes from https://www.moddb.com/downloads/cityscape-data-structure

## File format

| name | size | notes |
|------|------|-------|
| version code | UInt32 | currently grx=290, gry=325, g24=336 |
| side_size | UInt32 | no. bytes of side_block data |
| lid_size | UInt32 | no. bytes of lid_block data |
| aux_size | UInt32 | no. bytes of aux_block data |
| anim_size | UInt32 | no. bytes of anim data |
| palette_size | UInt32 | no. bytes of palette data (normally 768) |
| remap_size | UInt32 | no. bytes of remap_tables data (normally 65536) |
| remap_index_size | UInt32 | no. bytes of remap_index data (normally 1024) |
| object_info_size | UInt32 | no. bytes of object_info data |
| car_size | UInt32 | no. bytes of car_info data |
| sprite_info_size | UInt32 | no. bytes of sprite_info data |
| sprite_graphics_size | UInt32 | no. bytes of sprite_graphics data |
| sprite_numbers_size | UInt32 | no. bytes of sprite_numbers data |
| side_block | side_size | |
| lid_block | lid_size | |
| aux_block | aux_size | |
| anim | anim_size | |
| palette | palette_size | 3 bytes per colour RGB, each between 0 and 63 |
| remap_tables | remap_size | |
| remap_index | remap_index_size | |
| object_info | object_info_size | |
| car_info | car_size | |
| sprite_info | sprite_info_size | |
| sprite_graphics | sprite_graphics_size | |
| sprite_numbers | sprite_numbers_size | |

## Block Data

The block face data is stored in 3 areas :

```
char *side_block;
char *lid_block;
char *aux_block; 
```

Each of these stores between 0 and 256 64x64 block faces (in raw format).
The total number of faces among all 3 is limited to 384. (e.g. 224 + 96 + 64)
`side_block` stores the side block faces.
`lid_block` stores the lid block faces.
`aux_block` stores the auxiliary block faces ( for animation ).

The areas are actually not clearly delimited: all blocks are stored one after
the other without distinction.
The blocks are stored as a bitmap which is 256 pixels wide, so:
 - the bytes [0,63] are the first line of the first sprite
 - the bytes [64,127] are the first line of the second sprite
 - the bytes [128,191] are the first line of the third sprite
 - the bytes [192,255] are the first line of the fourth sprite
 - the bytes [256,319] are the second line of the first sprite
 - etc...

If the number of side sprites is not a multiple of 4, the first lid sprite will
be on the same line of blocks as the last side sprite.

So the simplest approach is to extract all blocks and then split the result.
Because the areas in the file containing the sides and the lids (resp. the lids
and the aux) can overlap.

### Synchronous Animation

The animation data is stored in 2 areas:

```
UInt8 num_anims;
loaded_anim *anims;
```

`num_anims`: a single-byte count, the number of animated blocks.

`anims`: a list of `num_anims` `loaded_anim` structures:
```
typedef struct  {
	UInt8 block;
    UInt8 which; 
	UInt8 speed;
	UInt8 frame_count;
	UInt8 frame[];
} loaded_anim;
```

where we have:
 - `block` - the block number in question
 - `which` – the area type ( 0 for side, 1 for lid )
 - `speed` - the number of game cycles to display each frame
 - `frame_count` - the number of auxiliary frames
 - `frame[]` - an array of block numbers -> these refer to aux_block.

When displaying the animation, the original side/lid_block face is shown first, then it is replaced by each of the specified aux_block faces in turn, until the animation repeats by going back to the first frame.

### Object Info

Object type description information is stored in a list which is appended to the block data on disk. For each object type, the following variable-length information record is stored :

```
typedef struct {
	Fix32 width, height, depth;
	UInt16 spr_num;
	UInt16 weight;
	UInt16 aux;
	Int8 status;
	UInt8 num_into;
	UInt16 into[MAX_INTO]; 
} object_info_struct;
```

Here, `width`, `height` and `depth` store the dimensions of the object with respect to collision checking. Note that width is for x, height is for y and depth is for z. These are Fix32 values but only the high end is stored in the file - this is shifted to make a correct Fix32 when the data is loaded. Hence, the value in the file is simply a pixel count ( to the resolution of 64 pixels per block ).

`spr_num` is the first sprite number offset for this object. Note that this number is relative to the first object sprite in the sprites file. However, it is converted to an absolute sprite number on loading. Subsequent sprites for this object will be found at subsequent sprite numbers. There will normally be 15 sprites for each object.
`aux` is an auxiliary word whose meaning depends on the type of object.
`weight` is a weight descriptor for the object.
`status` is a status descriptor for the object, which determines how it behaves. The meanings of the status descriptor are :

| status | meaning |
|--------|---------|
| 0 | normal |
| 1 | ignorable ( can drive over ) |
| 2 | smashable ( breaks on landing ) |
| 3 | invisible |

`num_into` is how many other objects this object breaks into when damaged.
`into` is a list of `num_into` object type codes, defining the objects which this object can break into.

#### Animated Objects

Animated objects are a special case. They cannot be involved in collisions and are there for graphical effect only. The same data structure is used as for objects, with the following differences :
`height` stores the number of game cycles per frame
`width` stores the number of frames
`depth` stores a life descriptor ( 0 for infinite, non-zero n for n animation cycles )
The animation works by displaying each of the desired frames in turn for the desired number of game cycles, then returning to the first frame after the last one.
If the life descriptor is set to 0 then the animation repeats indefinitely. If it is set to a non-zero number then the animation is played for that number of times and then the object destroys itself.

### Car Info

Car type information is stored in a list which is also stored in the style file. The following type definitions are used :

```
typedef struct {
	Int16 rpx, rpy;
	Int16 object
	Int16 delta
} door_info_struct;	

typedef struct {
	Int16 width, height, depth;
	Int16 spr_num;
	Int16 weight;	
	Int16 max_speed, min_speed;
	Int16 acceleration, braking;
	Int16 grip, handling;
    hls_info_struct remap24[12];
	UInt8 remap8[12];
    UInt8 vtype;
	UInt8 model;
	UInt8 turning;
	UInt8 damageable;
	UInt16 value[4];
	Int8 cx,cy;
	Int32 moment;
	float rbp_mass;
	float g1_thrust;
	float tyre_adhesion_x, tyre_adhesion_y;
	float handbrake_friction;
	float footbrake_friction;
	float front_brake_bias;
	Int16 turn_ratio;
	Int16 drive_wheel_offset;
	Int16 steering_wheel_offset;
	float back_end_slide_value;
	float handbrake_slide_value;
	UInt8 convertible;
	UInt8 engine;
	UInt8 radio;
	UInt8 horn;
	Uint8 sound_function;
	Uint8 fast_change_flag;
	Int16 doors;
	door_info_struct door [MAX_DOORS]	
} car_info_struct;
```

Here, `width`, `height` and `depth` store the dimensions of the car with respect to collision checking. Note that width is for x, height is for y and depth is for z. The value in the file is simply a pixel count ( to the resolution of 64 pixels per block ).
`spr_num` is the first sprite number offset for this car. Note that this number is relative to the first car sprite in the sprites file. However, it is converted to an absolute sprite number on loading. 
`weight` is a weight descriptor for the car.
`max_speed` and `min_speed` are descriptors of the maximum and minimum possible speeds for the car.
`acceleration`, `braking`, `grip`, `handling` and `turning` are descriptors of these characteristics of the car.
`damageable` reflects how easily the car can sustain damage
`vtype` is a descriptor of the type of car / vehicle. The meanings are :

| vtype | meaning |
|-------|---------|
| 0 | bus |
| 1 | front of juggernaut |
| 2 | back of juggernaut |
| 3 | motorcycle |
| 4 | standard car |
| 8 | train |

`model` is a sub-type within `vtype` for cars which holds an identifier for the model of car.
`remap` ( 8 - bit ) is an array of 12 remap numbers (for 8 bit) and hls remap infos (for 24 bit).
Remaps 1-6 are used for sprayshops. Remaps 7-12 are used for randomly generated dummy cars of this type ( along with the default remap which is stored with the car ). 
`value` is the monetary value of the car in the GTA mission, in 1000s of $. There are 4 value entries for the 4 cranes.
`cx`, `cy` is the pixel co-ordinates of the centre of mass of the car, relative to the graphical centre.
`moment` is the moment of inertia of the car.
`rbp_mass` is the total mass of the car.
`g1_thrust` is the ratio for 1s gear ( only one gear now ).
`handbrake_friction` is the friction of the handbrake.
`footbrake_friction` is the friction of the footbrake.
`front_brake_bias` is the front bias of braking.
`turn_ratio` is the turn ratio of the car.
`drive_wheel_offset`, `steering_wheel_offset`, `back_end_slide_value` and `handbrake_slide_value` are more handling controls.
`engine` is the engine type of the car ( for sound effects ).
`convertible` is 1 if the car is a convertible, else 0.
`radio` is the radio listening type of the car.
`horn` is the horn type of the car.
`sound_function` and `fast_change_flag` are for audio information.

`num_doors` is the number of opening doors which this car has. `door` is then a list of `num_doors` door info structures.
For each door, `(rpx,rpy)` is the relative position where a pedestrian must stand to enter / exit the car via that door. `delta` is the delta number of that door. `object` is the object type number of the door - this refers to an object info structure (see  ). 

#### floats

Note that all of the `float` quantities in the car data are stored in the file as 32-bit fixed point values ( with 16 bits after the point ). This is to aid Playstation compatibility. They are converted to float when they are loaded into the game.

## Sprites

### Sprite Info
The sprite info part of the style file contains `num_sprites` variable-sized `sprite_info` structures, which are described by the following type definitions :

```
typedef struct {
	UInt16 size;		// bytes for this delta
	Char *ptr;
} delta_info_struct;

typedef struct {
	UInt8	w;		// width of the sprite in pixels (2-64, even)
	UInt8	h;		// height of the sprite in pixels (1-254)
	UInt8	delta_count;	// number of deltas stored for this sprite (0-32)
	UInt8	ws;		// scaling flag ( valid on consoles only )
	UInt16	size;		// bytes per frame, i.e. w x h
	Char   *ptr;
	delta_info_struct delta[MAX_DELTAS];
} sprite_info_struct;
```

There is one `sprite_info_struct` for each different sprite graphic. Each contains information on a variable number of deltas, the number being given by `delta_count`. These deltas are small, variable sized, graphic changes which can be applied in any combination to this sprite. Deltas for one sprite cannot be applied to another sprite.

In the file, the `ptr` items in `sprite_info_struct` and `delta_info_struct` are 32 bits offsets to the actual position in memory of the graphic for the sprite or delta starting from the beginning of the `sprite_graphics` section.

#### Deltas
Deltas have the following format : 

`offset` ( 2 bytes )
`length` ( 1 byte )
`data` ( length bytes )

This is repeated as many times as is necessary to represent the differences between the original sprite and the changed one which the delta is for.
Note that the offset is always relative to the last position used (initially zero, includes the 'progress' caused by the length of the data).

The offset can be seen as (dx, dy) applied in the 256x256 page or as a UInt16 as it is strictly equivalent.

So for example, the decimal values 10 80 3 24 24 24 253 0 3 25 25 25 corresponds to 2 deltas:
 - first setting pixels (10, 80), (11, 80), (12, 80) to 24
 - second setting pixels (10, 81), (11, 81), (12, 81) to 25

### 24-bit Remaps

The format used for storing 24-bit remaps is :

```
typedef struct {
	Int16 h, l, s;
} hls_info_struct;
```

This represents a hue/lightness/saturation remap as used in Photoshop. However, if the hue value is greater than 1000 then the remap is loaded from a file instead of calculating it. The filename is remapXXX.tga, where X is hue-1000.

### Sprite Graphics

The sprite graphics part of the file contains the actual graphics for the sprites. It stores each sprite followed by all of the deltas for that sprite ( if any ). The order is the same as the order of `sprite_info_struct` records.

#### Sprites

Sprites are stored in a byte-per-pixel format, in row-major order, i.e. row 0 then row 1 then row 2, etc. They are not compressed at all.

#### Size Reduction
Sprites are stored in the smallest possible rectangle - the program which generates sprite data must enforce this. If any deltas are larger than the original sprite then the sprite must be stored in a rectangle big enough to hold these deltas and no bigger.

### Sprite Numbers
Sprite numbers data is also stored in the style file. This is a list of numbers which is used by the game to reference particular sprite types. The format is :

```
typedef struct {
	UInt16 SPR_ARROW;
	UInt16 SPR_DIGITS;
	UInt16 SPR_BOAT;
	UInt16 SPR_BOX;
	UInt16 SPR_BUS;
	UInt16 SPR_CAR;
	UInt16 SPR_OBJECT;
	UInt16 SPR_PED;
	UInt16 SPR_SPEEDO;
	UInt16 SPR_TANK;
	UInt16 SPR_TRAFFIC_LIGHTS;
	UInt16 SPR_TRAIN;
	UInt16 SPR_TRDOORS;
	UInt16 SPR_BIKE;
	UInt16 SPR_TRAM;
	UInt16 SPR_WBUS;
	UInt16 SPR_WCAR;
	UInt16 SPR_EX;
	UInt16 SPR_TUMCAR;
	UInt16 SPR_TUMTRUCK;
	UInt16 SPR_FERRY;
} sprite_numbers_struct;
```

Each of these numbers stores the number of sprites of that particular type. The number can be zero if there are no sprites of that type in the style.

## `remap_tables` ( 8-bit only )
`remap_tables` is the area of the style file which stores the colour remap information for the style. It stores 256 separate remap tables. Each remap table contains 256 bytes. These bytes represent a re-ordering of the original colour palette.  To remap a sprite. Each pixel’s colour is used as an index into the relevant remap table.
Remaps can be applied to cars, pedestrians, objects and tile lids. Unused remap tables will store a copy of the original palette.
Tiles can use any remap from 0-255 but sprites (cars, peds, objects ) can use only 0-127.

## `remap_index`
`remap_index` is a table which stores the table numbers of the 3 remaps which can be applied to each lid tile. The structure is :

```
UInt8 remap_index[256][4];
```

This is an array of 256 sub-arrays, where each sub-array refers to 1 tile. In the sub-array, the elements store remap table numbers. Element 0 always stores 0 ( meaning no remap ). The other elements can store any number from 0-255.

## `palette_index` ( 24-bit only )

Rather than store all of the generated palettes, only unique palettes are stored. A tool called palcut takes the original palette data and generates a minimal palette set plus an index, which maps the old palette numbers onto their position in the new set. This index is stored in the 24-bit style file as `palette_index`.

