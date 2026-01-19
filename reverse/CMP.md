The CMP file format is a low-flexibility file format with a fixed structure used for storing map data.

Most of the information in this page comes from https://www.moddb.com/downloads/cityscape-data-structure

 == File format

| name | size | notes |
-----------------------
| version code | UInt32 | 331 - 0x0000014b |
| style number | UInt8 | style number xxx means that the style file to be used for this map is stylexxx.gry
| sample number | UInt8 | sample file number xxx means that the sample file to be used for this map is samxxx.sam
| reserved | Uint16 | |
| route_size | UInt32 | no. bytes of route data |
| object_pos size | UInt32 | no. bytes of object_pos data |
| column_size | UInt32 | |
| block_size | UInt32 | |
| nav_data_size | UInt32 | no. bytes of nav_data (navigational data ) |
| base | 256x256x4 | |
| column | column_size | |
| block | block_size | |
| object_pos | object_pos_size | |
| route | route_size | |
| location_data | 3x6x6 | |
| nav_data | nav_data_size | |

 == Internal representation

Although this is not how the data is stored into the file, it helps understand how the original GTA game represents the data in memory to understand the file format.

The internal data is often referenced as a fixed-size 256x256x6 grid, where the dimensions correspond to x, y, and z coordinates for map blocks.

```
typedef struct {
	UInt16 type_map;
	UInt8 type_map_ext;
	Char left, right, top, bottom, lid;
} block_info;

block_info city_scape[6][256][256];
```

`type_map` : this is a bitmap with the following structure:

| bits      | content |
|-----------|---------|
| bits 0-3  | direction bits (on road, indicate vehicle direction) 0=up, 1=down, 2=left, 3=right |
| bits 4-6  | block type (0=air, 1=water, 2=road, 3=pavement, 4=field, 5=building, 6-7=unused |
| bit 7     | flat (1=yes, 0=no) - used for road signs, only left and top are drawn, can be walked/driven through |
| bits 8-13 | slope type (0=none, 1-2=up, 3-4=down, 5-6=left, 7-8=right, 9-16=up, 17-24=down, 25-32=left, 33-40=right, 41-44=up,down,left,right, 45=?) |
| bit 14-15 | rotation of lid (0 = normal, 1 = 90°, 2=180°, 3=270°) |


`type_map_ext`: this is a bitmap with the following structure:

| bits      | content |
|-----------|---------|
| bits 0-2  | traffic light bits 1,2,3 (3 bit code for traffic light info)
| bits 3-4  | remap  (0, 1, 2, 3)
| bit 5     | flip top & bottom faces ( 1 = yes, 0=no)
| bit 6     | flip left & right faces ( 1 = yes, 0=no)
| bit 7     | railway ( 1 = yes, 0 = no )

`left, right, top, bottom, lid`: these bytes store the face information - each byte stores a value which indicates the correct graphic square to use for that face. Multiply by 64x64 to get the offset into the face graphics file. For left, right, top and bottom, this is the block side graphics. For lid, it is the block lid graphics.  A value of zero indicates no face.

Using `[z][y][x]` order, position `[0][0][0]` is the highest left top corner of the world.
`[5][n][n]` is the lowest level. Note that at this level only lids are visible.

 == Map data

 === `base`, `column` and `block`

The map data is converted to a compressed format to be stored into the file, to reduce memory usage.

After compression, the map is stored in 3 data areas :

```
UInt32 base [256][256];
UInt16 column[];
block_info block[];
```

`base`: a 2D array of 32-bit unsigned ints, where each int stores a byte offset into column. - i.e. it stores a pointer to the column which sits at that square on the ground.

`column`: a variable length array of shorts (16-bit unsigned integers). For each column, the format is:
```
typedef struct {
	Uint16 height;
	UInt16 blockd[];
} col_info;
```

Here,  height is the minimum array index for the column (i.e. 6-N where N is the number of blocks in the column). Note that this height may include a top block which has a type but no graphic.
`blockd` is a variable length array of block numbers, with `blockd[0]` being the ground-level one. Each block number is a reference to the block data stored in block.

`block`: a variable length array of `block_info` structures, containing every distinct combination of faces & types required for the level.

 === `object_pos`
Objects positions are stored in a list of:
```
typedef struct {
	UInt16 x,y,z;
	Uint8 type;
	UInt8 remap;
	Ang16 rotation;
	Ang16 pitch;
	Ang16 roll;
} object_pos_struct;
```

Here, `(x,y,z)` is the position of the object in the world, stated in world co-ordinates (but not fixed point).
`type` is the object type - a code between zero and the maximum number of object types which gives an index into the object info.
`remap` is a remap table number, indicating a remap which is to be applied to this object ( 0 for none ). If remap is >=128 then the item is a car, not an object.
`rotation`, `pitch` and `roll` are the initial rotation, pitch and roll angles of the object. They are of type `Ang16`, which is a two-byte positive integer between zero and `MAX_ANGLE`, where `MAX_ANGLE` is the equivalent of 360°.
There is one entry in this list for each distinct object which is to be present in the world when the game starts.

 === `route`
Routefinder information.

An individual route is a list of between 1 and 50 vertices, where each vertex has the following format :

```
typedef struct {
	Uint8 x, y, z;
} file_vertex_struct;
```

The route part of the compressed map file can contain any number of routes. It is of the format :

```
UInt8 num_vertices;
UInt8 route_type;  // 255 for police
file_vertex_struct vertex_list [];
```

These 3 fields are repeated for each route.

As well as the police routes, the route area can also contain roadblock routes. These are variable length lists of co-ordinates of junctions which are relevant to some particular roadblock. Each roadblock route is numbered. There can be up to 254 of them per map (0-253). This number is referenced from within the object position info.

// TODO: understand how roadbloacks work and their relation with traffic lights.

 === `location_data`
Information about particular locations: police stations, hospitals and fire stations.

This is in the following format :

```
typedef struct {
	file_vertex_struct police_station [6];
	file_vertex_struct hospital [6];
	file_vertex_struct unused [6];
	file_vertex_struct unused [6];
	file_vertex_struct fire_station [6];
	file_vertex_struct unused [6];
} location_data_struct;
```

This stores (x,y,z) location information for the accident services bases. Unused bases will have a location of (0,0,0). Note that this means that no base can actually be at (0,0,0). There must be at least one and no more than six bases for each service.

There are some unused locations defined in the data files from the games, but it's unclear why. Removing them doesn't seem to have any effect (but maybe it impacts some mission?).

 === `nav_data`

Navigational data. This consists of the name of each area of the map, where it is, and how to say it.
Each named area must be a rectangle. Overlapping of areas is allowed - the smaller area always has priority. Every block of the map must be within at least one named area. A sequence of structures of the following format is stored in the `nav_data` area of the map file : 

```
typedef struct {
	UInt8 x,y;		// top left of the rectangle, array co-ords
	Uint8 w,h;		// width & height of the rectangle, in blocks
	UInt8 sam;		// sample number to say the name
	Char name[30];		// null-terminated string - text of the name
} nav_data_struct
```

Also relevant to the navigational data is the sample file number, stored in the map file header. This defines which sample file is to be loaded with this map - all sample numbers in the nav_data refer to this sample file.

The `sam` number means that the name will be in the sound sample 99 + `sam` in LEVELxxx.RAW. It seems that 0 corresponds to sam=21?! (Nixon Island in level001)
