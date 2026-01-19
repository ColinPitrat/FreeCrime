#!/usr/bin/env python3
import argparse
import struct
import os
import sys

class CMPReader:
    def __init__(self, data, offset=0):
        self.data = data
        self.offset = offset

    def read_uint8(self):
        val = struct.unpack_from('<B', self.data, self.offset)[0]
        self.offset += 1
        return val

    def read_int8(self):
        val = struct.unpack_from('<b', self.data, self.offset)[0]
        self.offset += 1
        return val

    def read_uint16(self):
        val = struct.unpack_from('<H', self.data, self.offset)[0]
        self.offset += 2
        return val

    def read_uint32(self):
        val = struct.unpack_from('<I', self.data, self.offset)[0]
        self.offset += 4
        return val

    def read_string(self, length):
        val = self.data[self.offset : self.offset + length]
        self.offset += length
        val = val.split(b'\x00', 1)[0]
        return val.decode('latin-1', errors='replace')

    def skip(self, amount):
        self.offset += amount

class BlockInfo:
    def __init__(self, type_map, type_map_ext, left, right, top, bottom, lid):
        self.type_map = type_map
        direction = type_map & 0xf
        # If it's an intersection, it's valid to combine, for example, right & down
        self.direction = "none" if direction == 0 else "up" if direction == 1 else "down" if direction == 2 else "left" if direction == 4 else "right" if direction == 8 else f"invalid:{direction}"
        block_type = type_map & 0x70 >> 4
        self.block_type = "air" if block_type == 0 else "water" if block_type == 1 else "road" if block_type == 2 else "pavement" if block_type == 3 else "field" if block_type == 4 else "building" if block_type == 5 else f"invalid:{block_type}"
        self.flat = type_map & 0x80 != 0
        # TODO: clarify slope_type
        # There are 6 bits, so 64 possible values.
        # Documentation suggests multiple values for up (resp. down, left, right)
        # but what does this mean exactly?
        self.slope_type = type_map & 0x3F00 >> 8
        lid_rotation = type_map & 0xC000 >> 14
        self.lid_rotation = f"{90*lid_rotation}°"

        self.type_map_ext = type_map_ext
        self.traffic_light = type_map_ext & 0x7
        self.remap = type_map_ext & 0x18 >> 3
        self.flip_y = (type_map_ext & 0x20) != 0
        self.flip_x = (type_map_ext & 0x40) != 0
        self.railway = (type_map_ext & 0x80) != 0

        self.left = left
        self.right = right
        self.top = top
        self.bottom = bottom
        self.lid = lid

    @classmethod
    def from_bytes(cls, reader):
        tm = reader.read_uint16()
        tme = reader.read_uint8()
        l = reader.read_uint8()
        r = reader.read_uint8()
        t = reader.read_uint8()
        b = reader.read_uint8()
        lid = reader.read_uint8()
        return cls(tm, tme, l, r, t, b, lid)

    def __repr__(self):
        return (f"Block(TM={self.type_map:#06x}, TME={self.type_map_ext:#04x}, "
                f"direction={self.direction}, block_type={self.block_type}, flat={self.flat}, slope={self.slope_type}, lid_rotation={self.lid_rotation}, "
                f"traffic_light={self.traffic_light}, remap={self.remap}, flip_y={self.flip_y}, flip_x={self.flip_x}, railway={self.railway} "
                f"Faces=[L:{self.left}, R:{self.right}, T:{self.top}, B:{self.bottom}, Lid:{self.lid}])")

class ObjectPos:
    def __init__(self, x, y, z, obj_type, remap, rotation, pitch, roll):
        self.x = x
        self.y = y
        self.z = z
        self.type = obj_type
        self.car = remap >= 128
        if self.car:
            remap -= 128
        self.remap = remap
        self.rotation = rotation
        self.pitch = pitch
        self.roll = roll

    @classmethod
    def from_bytes(cls, reader):
        x = reader.read_uint16()
        y = reader.read_uint16()
        z = reader.read_uint16()
        otyp = reader.read_uint8()
        remap = reader.read_uint8()
        rot = reader.read_uint16()
        pit = reader.read_uint16()
        rol = reader.read_uint16()
        return cls(x, y, z, otyp, remap, rot, pit, rol)

    def __repr__(self):
        return (f"Object(Pos=({self.x},{self.y},{self.z}), "
                f"Car={self.car}, Type={self.type}, Remap={self.remap}, "
                f"Rot=({self.rotation},{self.pitch},{self.roll}))")

class Route:
    def __init__(self, route_type, vertices):
        self.route_type = route_type
        self.vertices = vertices

    @classmethod
    def from_bytes(cls, reader):
        num_v = reader.read_uint8()
        r_type = reader.read_uint8()
        verts = []
        for _ in range(num_v):
            x = reader.read_uint8()
            y = reader.read_uint8()
            z = reader.read_uint8()
            verts.append((x, y, z))
        return cls(r_type, verts)

    def __repr__(self):
        return f"Route(Type={self.route_type}, Vertices={len(self.vertices)})"

class NavZone:
    def __init__(self, x, y, w, h, sam, name):
        self.x = x
        self.y = y
        self.w = w
        self.h = h
        self.sam = sam
        self.name = name

    @classmethod
    def from_bytes(cls, reader):
        x = reader.read_uint8()
        y = reader.read_uint8()
        w = reader.read_uint8()
        h = reader.read_uint8()
        sam = reader.read_uint8()
        name = reader.read_string(30)
        return cls(x, y, w, h, sam, name)

    def __repr__(self):
        sample_num = (self.sam - 1) if self.sam > 0 else 20
        sample_file = f"LEVEL00x_01{sample_num:02}.WAV"
        return f"NavZone(Name='{self.name}', Rect=({self.x},{self.y},{self.x+self.w},{self.y+self.h}), Sample={sample_file})"

class CMPFile:
    def __init__(self):
        self.header = {}
        self.base = [] # 256x256 array of offsets
        self.columns_data = b'' # Raw column bytes to access via offsets
        self.blocks = []
        self.objects = []
        self.routes = []
        self.locations = {}
        self.nav_zones = []

    def parse(self, filepath):
        with open(filepath, 'rb') as f:
            data = f.read()

        reader = CMPReader(data)

        # 1. Header
        self.header['version'] = reader.read_uint32()
        self.header['style'] = reader.read_uint8()
        self.header['sample'] = reader.read_uint8()
        self.header['reserved'] = reader.read_uint16()
        self.header['route_size'] = reader.read_uint32()
        self.header['object_pos_size'] = reader.read_uint32()
        self.header['column_size'] = reader.read_uint32()
        self.header['block_size'] = reader.read_uint32()
        self.header['nav_data_size'] = reader.read_uint32()

        # 2. Base
        # 256x256 = 65536 entries
        # Storing as flat list for memory efficiency, can index with y*256+x
        self.base = [reader.read_uint32() for _ in range(256*256)]

        # 3. Column
        # Store raw data for now, as base points into it.
        # We can parse specific columns if needed or just store size.
        start_col = reader.offset
        self.columns_data = data[start_col : start_col + self.header['column_size']]
        reader.skip(self.header['column_size'])

        # 4. Block
        # block_size bytes. Each block is 8 bytes.
        num_blocks = self.header['block_size'] // 8
        for _ in range(num_blocks):
            self.blocks.append(BlockInfo.from_bytes(reader))

        # 5. Object Pos
        start_obj = reader.offset
        end_obj = start_obj + self.header['object_pos_size']
        while reader.offset < end_obj:
            self.objects.append(ObjectPos.from_bytes(reader))

        # 6. Route
        start_route = reader.offset
        end_route = start_route + self.header['route_size']
        while reader.offset < end_route:
            self.routes.append(Route.from_bytes(reader))

        # 7. Location Data
        # Fixed 108 bytes
        # struct { police[6], hospital[6], unused[6], unused[6], fire[6], unused[6] }
        # each entry is 3 bytes (x,y,z)
        def read_loc_group(reader):
            return [(reader.read_uint8(), reader.read_uint8(), reader.read_uint8()) for _ in range(6)]

        self.locations['police'] = read_loc_group(reader)
        self.locations['hospital'] = read_loc_group(reader)
        self.locations['unused1'] = read_loc_group(reader)
        self.locations['unused2'] = read_loc_group(reader)
        self.locations['fire'] = read_loc_group(reader)
        self.locations['unused3'] = read_loc_group(reader)

        # 8. Nav Data
        start_nav = reader.offset
        end_nav = start_nav + self.header['nav_data_size']
        while reader.offset < end_nav:
            self.nav_zones.append(NavZone.from_bytes(reader))

    def get_columns(self):
        pos = 0
        columns = []
        r = CMPReader(self.columns_data, 0)
        while pos < len(self.columns_data):
            offset = pos
            height = r.read_uint16()
            pos += 2
            blocks = []
            for _ in range(6 - height):
                blocks.append(r.read_uint16())
                pos += 2
            columns.append({'offset': offset, 'height': height, 'blocks': blocks})
        return columns

    def get_column_info(self, offset):
        # Helper to parse a column at a given offset in columns_data
        if offset >= len(self.columns_data):
            return None

        r = CMPReader(self.columns_data, offset)
        height = r.read_uint16()
        # height is minimum array index (6-N).
        # Blocks are from level 'height' to 5.
        # Count = 6 - height.
        count = 6 - height
        blocks = []
        for _ in range(count):
            blocks.append(r.read_uint16())
        return {'offset': offset, 'height': height, 'blocks': blocks}

    def display_sections(self):
        print("== Section Sizes ==")
        print(f"Header Version: {self.header['version']} ({self.header['version']:#x})")
        print(f"Base: {256*256*4} bytes")
        print(f"Column: {self.header['column_size']} bytes")
        print(f"Block: {self.header['block_size']} bytes")
        print(f"Object Pos: {self.header['object_pos_size']} bytes")
        print(f"Route: {self.header['route_size']} bytes")
        print(f"Location Data: {3*6*6} bytes")
        print(f"Nav Data: {self.header['nav_data_size']} bytes")
        print()

    def display_map_info(self):
        print("== Map Information ==")
        # Not sure whether a style file of 0 would be valid, but aligning the logic with sample.
        # We never have a sample file in GTA1 data, sounds are in LEVELxxx.
        style_file = f"STYLE{self.header['style']:03}.GRY" if self.header['style'] != 0 else None
        print(f"Style file: ")
        sample_file = f"SAM{self.header['style']:03}.SAM" if self.header['sample'] != 0 else None
        print(f"Sample file: {self.header['sample']}")
        print(f"Base Grid: 256x256")

        # Basic stats on base offsets
        valid_offsets = [x for x in self.base if x < self.header['column_size']]
        print(f"Valid Base Offsets: {len(valid_offsets)} / {len(self.base)}")

        print(f"Total Blocks defined: {len(self.blocks)}")
        if self.blocks:
             print(f"First Block: {self.blocks[0]}")
             print(f"Last Block: {self.blocks[-1]}")
        print()

    def display_blocks_info(self):
        print("== Blocks Information ==")
        for (i, block) in enumerate(self.blocks):
            print(f"Block #{i}: {block}")
        print()

    def display_objects(self):
        print(f"== Objects ({len(self.objects)}) ==")
        for i, obj in enumerate(self.objects):
            print(f"#{i}: {obj}")
        print()

    def display_routes(self):
        print(f"== Routes ({len(self.routes)}) ==")
        for i, route in enumerate(self.routes):
            print(f"#{i}: {route}")
            # print vertices if needed?
        print()

    def display_locations(self):
        print("== Locations ==")
        for key, locs in self.locations.items():
            valid_locs = [l for l in locs if not (l[0]==0 and l[1]==0 and l[2]==0)]
            print(f"{key.capitalize()}: {len(valid_locs)} entries")
            for l in valid_locs:
                print(f"  - ({l[0]}, {l[1]}, {l[2]})")
        print()

    def display_nav(self):
        print(f"== Navigation Data ({len(self.nav_zones)}) ==")
        for i, nav in enumerate(self.nav_zones):
            print(f"#{i}: {nav}")
        print()

    def find_block(self, block_id):
        columns = dict()
        for i, c in enumerate(self.get_columns()):
            for j, b in enumerate(c['blocks']):
                if b == block_id:
                    print(f'Block {block_id} found in column {i} at height {j}')
                    columns[c['offset']] = c
        for i, col in enumerate(self.base):
            if col in columns.keys():
                y = i//256
                x = i%256
                print(f'Column {columns[col]} used at {x},{y} (base[{y}][{x}])')


def main():
    parser = argparse.ArgumentParser(description="Decode GTA CMP map files.")
    parser.add_argument("filename", help="Path to the CMP file")
    parser.add_argument("--sections", action="store_true", help="Display section sizes")
    parser.add_argument("--map", action="store_true", help="Display map info")
    parser.add_argument("--blocks", action="store_true", help="Display blocks info")
    parser.add_argument("--objects", action="store_true", help="Display objects")
    parser.add_argument("--routes", action="store_true", help="Display routes")
    parser.add_argument("--locations", action="store_true", help="Display locations")
    parser.add_argument("--nav", action="store_true", help="Display navigation data")
    parser.add_argument("--all", action="store_true", help="Display everything")
    parser.add_argument("--find_block", type=int, default=-1, help="Find where a block is used")

    args = parser.parse_args()

    if not os.path.exists(args.filename):
        print(f"Error: File '{args.filename}' not found.")
        sys.exit(1)

    cmp = CMPFile()
    try:
        cmp.parse(args.filename)
    except Exception as e:
        print(f"Error parsing file: {e}")
        sys.exit(1)

    # If no flags provided, show sections by default, or maybe help?
    # Prompt says "depending on flags passed ... displays various information".
    # I'll default to sections if nothing else is specified? Or just do nothing.
    # Usually better to show something. Let's show sections if no specific flag.

    show_any = any([args.sections, args.map, args.objects, args.routes, args.locations, args.nav, args.all])

    if args.all or args.sections or not show_any:
        cmp.display_sections()

    if args.all or args.map:
        cmp.display_map_info()

    if args.all or args.blocks:
        cmp.display_blocks_info()

    if args.all or args.objects:
        cmp.display_objects()

    if args.all or args.routes:
        cmp.display_routes()

    if args.all or args.locations:
        cmp.display_locations()

    if args.all or args.nav:
        cmp.display_nav()

    if args.find_block >= 0:
        cmp.find_block(args.find_block);

if __name__ == "__main__":
    main()
