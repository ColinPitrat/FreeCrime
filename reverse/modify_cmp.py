import struct
import argparse
import sys
import re

class CMPFile:
    def __init__(self, filepath):
        self.filepath = filepath
        with open(filepath, 'rb') as f:
            self.data = f.read()
        self.parse()

    def parse(self):
        offset = 0
        # Header
        fmt = '<I B B H I I I I I'
        size = struct.calcsize(fmt)
        values = struct.unpack(fmt, self.data[offset:offset+size])
        self.header = {
            'version_code': values[0],
            'style_number': values[1],
            'sample_number': values[2],
            'reserved': values[3],
            'route_size': values[4],
            'object_pos_size': values[5],
            'column_size': values[6],
            'block_size': values[7],
            'nav_data_size': values[8],
        }
        offset += size

        # Base
        base_size = 256 * 256 * 4
        self.raw_base = self.data[offset:offset+base_size]
        offset += base_size

        # Column
        col_size = self.header['column_size']
        self.raw_columns = self.data[offset:offset+col_size]
        self.map_offset_to_column = dict()
        self.columns = self.parse_columns(self.raw_columns)
        offset += col_size

        # We can only parse the base once we have the columns information as we
        # do the offset -> column translation.
        self.base = self.parse_base(self.raw_base)

        # Block
        blk_size = self.header['block_size']
        self.raw_blocks = self.data[offset:offset+blk_size]
        self.blocks = self.parse_blocks(self.raw_blocks)
        offset += blk_size

        # Object Pos
        obj_size = self.header['object_pos_size']
        self.raw_object_pos = self.data[offset:offset+obj_size]
        self.object_pos = self.parse_object_pos(self.raw_object_pos)
        offset += obj_size

        # Route
        route_size = self.header['route_size']
        self.raw_route = self.data[offset:offset+route_size]
        self.route = self.parse_route(self.raw_route)
        offset += route_size

        # Location Data
        # 3x6x6 bytes = 108 bytes
        loc_size = 108
        self.raw_location_data = self.data[offset:offset+loc_size]
        self.location_data = self.parse_location_data(self.raw_location_data)
        offset += loc_size

        # Nav Data
        nav_size = self.header['nav_data_size']
        self.raw_nav_data = self.data[offset:offset+nav_size]
        self.nav_data = self.parse_nav_data(self.raw_nav_data)
        offset += nav_size

        self.remaining = self.data[offset:]

    def parse_base(self, data):
        base = []
        fmt = '<' + 'I'*256
        stride = struct.calcsize(fmt)
        for i in range(256):
            base.append([self.offset_to_column(offset) for offset in struct.unpack_from(fmt, data, i*stride)])
        return base

    def parse_blocks(self, data):
        num_blocks = self.header['block_size'] // 8
        blocks = []
        pos = 0
        for _ in range(num_blocks):
            (type_map, type_map_ext, left, right, top, bottom, lid) = struct.unpack_from('<Hbbbbbb', data, pos)
            pos += 8
            block = { 'type_map': type_map, 'type_map_ext': type_map_ext, 'left': left, 'right': right, 'top': top, 'bottom': bottom, 'lid': lid }
            blocks.append(block)
        return blocks

    def column_to_offset(self, column):
        return self.columns[column]['offset']

    def offset_to_column(self, offset):
        return self.map_offset_to_column[offset]

    def parse_columns(self, data):
        columns = []
        pos = 0
        i = 0
        fmt = '<' + 'H'
        while pos < len(data):
            height = struct.unpack_from(fmt, data, pos)[0]
            offset = pos
            pos += 2
            blockd = []
            for blocks in range(height, 6):
                blockd.append(struct.unpack_from(fmt, data, pos)[0])
                pos += 2
            columns.append({'offset': offset, 'height': height, 'blockd': blockd})
            self.map_offset_to_column[offset] = i
            i += 1
        return columns

    def parse_object_pos(self, data):
        objects = []
        fmt = '<H H H B B H H H'
        stride = struct.calcsize(fmt)
        count = len(data) // stride
        for i in range(count):
            vals = struct.unpack_from(fmt, data, i * stride)
            obj = {
                'x': vals[0], 'y': vals[1], 'z': vals[2],
                'type': vals[3], 'remap': vals[4],
                'rotation': vals[5], 'pitch': vals[6], 'roll': vals[7]
            }
            objects.append(obj)
        return objects

    def pack_base(self):
        data = bytearray()
        fmt = '<I'
        for row in self.base:
            for block in row:
                data.extend(struct.pack(fmt, self.column_to_offset(block)))
        return data

    def pack_blocks(self):
        data = bytearray()
        for block in self.blocks:
            data.extend(struct.pack('<Hbbbbbb', block['type_map'], block['type_map_ext'], block['left'], block['right'], block['top'], block['bottom'], block['lid']))
        return data

    def pack_columns(self):
        data = bytearray()
        fmt = '<H'
        for column in self.columns:
            want_height = 6-len(column['blockd'])
            if column['height'] != want_height:
                col_height = column['height']
                print(f"ERROR: column.height == {col_height} != 6-len(column[blockd]) == {want_height}")
            data.extend(struct.pack(fmt, want_height))
            for i in reversed(range(want_height, 6)):
                data.extend(struct.pack(fmt, column['blockd'][5-i]))
        return data

    def pack_object_pos(self):
        data = bytearray()
        fmt = '<H H H B B H H H'
        for obj in self.object_pos:
            data.extend(struct.pack(fmt, obj['x'], obj['y'], obj['z'], obj['type'], obj['remap'], obj['rotation'], obj['pitch'], obj['roll']))
        return data

    def parse_route(self, data):
        routes = []
        offset = 0
        while offset < len(data):
            num_vertices = data[offset]
            route_type = data[offset+1]
            offset += 2
            vertices = []
            for _ in range(num_vertices):
                if offset + 3 > len(data): break
                x, y, z = struct.unpack_from('<B B B', data, offset)
                vertices.append({'x': x, 'y': y, 'z': z})
                offset += 3
            routes.append({'num_vertices': num_vertices, 'route_type': route_type, 'vertices': vertices})
        return routes

    def pack_route(self):
        data = bytearray()
        for r in self.route:
            data.append(len(r['vertices']))
            data.append(r['route_type'])
            for v in r['vertices']:
                data.extend(struct.pack('<B B B', v['x'], v['y'], v['z']))
        return data

    def parse_location_data(self, data):
        keys = ['police_station', 'hospital', 'unused1', 'unused2', 'fire_station', 'unused3']
        locs = {}
        offset = 0
        for key in keys:
            loc_list = []
            for _ in range(6):
                x, y, z = struct.unpack_from('<B B B', data, offset)
                loc_list.append({'x': x, 'y': y, 'z': z})
                offset += 3
            locs[key] = loc_list
        return locs

    def pack_location_data(self):
        data = bytearray()
        keys = ['police_station', 'hospital', 'unused1', 'unused2', 'fire_station', 'unused3']
        for key in keys:
            for v in self.location_data[key]:
                data.extend(struct.pack('<B B B', v['x'], v['y'], v['z']))
        return data

    def parse_nav_data(self, data):
        navs = []
        fmt = '<B B B B B 30s'
        stride = 35
        count = len(data) // stride
        for i in range(count):
            vals = struct.unpack_from(fmt, data, i * stride)
            name = vals[5].split(b'\x00')[0].decode('ascii', errors='ignore')
            nav = {
                'x': vals[0], 'y': vals[1], 'w': vals[2], 'h': vals[3],
                'sam': vals[4], 'name': name
            }
            navs.append(nav)
        return navs

    def pack_nav_data(self):
        data = bytearray()
        fmt = '<B B B B B 30s'
        for n in self.nav_data:
            name_bytes = n['name'].encode('ascii')
            # Pad with nulls
            if len(name_bytes) < 30:
                name_bytes += b'\x00' * (30 - len(name_bytes))
            name_bytes = name_bytes[:30]
            data.extend(struct.pack(fmt, n['x'], n['y'], n['w'], n['h'], n['sam'], name_bytes))
        return data

    def save(self, filepath):
        obj_data = self.pack_object_pos()
        self.header['object_pos_size'] = len(obj_data)

        route_data = self.pack_route()
        self.header['route_size'] = len(route_data)

        loc_data = self.pack_location_data()

        nav_data = self.pack_nav_data()
        self.header['nav_data_size'] = len(nav_data)

        self.raw_base = self.pack_base()
        self.raw_columns = self.pack_columns()
        self.raw_blocks = self.pack_blocks()

        with open(filepath, 'wb') as f:
            f.write(struct.pack('<I B B H I I I I I',
                self.header['version_code'], self.header['style_number'], self.header['sample_number'], self.header['reserved'],
                self.header['route_size'], self.header['object_pos_size'], self.header['column_size'], self.header['block_size'],
                self.header['nav_data_size']))

            f.write(self.raw_base)
            f.write(self.raw_columns)
            f.write(self.raw_blocks)
            f.write(obj_data)
            f.write(route_data)
            f.write(loc_data)
            f.write(nav_data)
            f.write(self.remaining)

def get_item(obj, key):
    if isinstance(obj, dict):
        return obj[key]
    if isinstance(obj, list):
        return obj[int(key)]
    return getattr(obj, key)

def set_val(obj, key, val):
    if isinstance(obj, dict):
        obj[key] = val
    elif isinstance(obj, list):
        obj[int(key)] = val
    else:
        setattr(obj, key, val)

def resolve_path(root, path):
    # path examples:
    # header.style_number
    # location_data.police_station[0].x
    # object_pos[10].type

    # Split by . but preserve brackets logic
    # Actually, simpler: replace [N] with .N and split by .
    # e.g. location_data.police_station[0].x -> location_data.police_station.0.x

    clean_path = path.replace('[', '.').replace(']', '')
    parts = clean_path.split('.')

    current = root
    for part in parts[:-1]:
        try:
            current = get_item(current, part)
        except (KeyError, IndexError, AttributeError) as e:
            print(f"Error resolving path '{path}': segment '{part}' failed. {e}")
            sys.exit(1)

    last_part = parts[-1]
    return current, last_part

def main():
    parser = argparse.ArgumentParser(description='Modify CMP files')
    parser.add_argument('input_file', help='Input CMP file')
    parser.add_argument('--output', '-o', help='Output CMP file (default: overwrite input)')
    parser.add_argument('--set', '-s', action='append', help='Set field value, e.g. location_data.police_station[0].x=100')
    parser.add_argument('--print', '-p', action='append', help='Print field value, e.g. header.style_number')
    parser.add_argument('--print_slopes', '-P', action='store_true', help='Print a map of the slopes')
    parser.add_argument('--generate', '-g', help='Generate a test map (with only roads)')

    args = parser.parse_args()

    cmp_file = CMPFile(args.input_file)

    if args.print:
        for p in args.print:
            parent, key = resolve_path(cmp_file, p)
            try:
                val = get_item(parent, key)
                print(f"{p} = {val}")
            except Exception as e:
                print(f"Error reading {p}: {e}")

    if args.print_slopes:
        for z in range(6):
            print(f"Layer {z}:")
            for y in range(256):
                for x in range(256):
                    column = cmp_file.base[y][x]
                    blockd = cmp_file.columns[column]['blockd']
                    slope = 0
                    if z < len(blockd):
                        slope = (cmp_file.blocks[blockd[z]]['type_map'] & 0x3F00) >> 8
                    print(f"{slope}",end=",")
                print()
            print()

    if args.set:
        for s in args.set:
            if '=' not in s:
                print(f"Invalid set format: {s}. Expected field=value")
                continue
            path, value = s.split('=', 1)
            parent, key = resolve_path(cmp_file, path)

            # Determine type of existing value
            try:
                current_val = get_item(parent, key)
                target_type = type(current_val)
                if target_type == int:
                    new_val = int(value)
                elif target_type == float:
                    new_val = float(value)
                elif target_type == str:
                    new_val = value
                else:
                    # fallback
                    if value.isdigit():
                        new_val = int(value)
                    else:
                        new_val = value

                set_val(parent, key, new_val)
                print(f"Set {path} to {new_val}")
            except Exception as e:
                print(f"Error setting {path}: {e}")
                sys.exit(1)

        output_path = args.output if args.output else args.input_file
        cmp_file.save(output_path)
        print(f"Saved to {output_path}")

    # Very specific logic: generate a test map copying the same column
    # everywhere from the provided coordinates.
    # Example for NYC.CMP, can be used with --generate=32,137 on the range
    # 100-120 to generate a huge "road area" around the player starting
    # position.
    if args.generate:
        coords = args.generate.split(',')
        if len(coords) != 2:
            print(f"Error parsing coordinates: {args.generate}")
            sys.exit(1)
        x, y = int(coords[0]), int(coords[1])
        if not args.output:
            print(f"Error: --generate must be used with --output. I don't want to fully overwrite the real map!")
            sys.exit(1)
        to_copy = cmp_file.base[y][x]
        #for i in range(256):
        for i in range(100, 120):
        #    for j in range(256):
            for j in range(100, 120):
                cmp_file.base[i][j] = to_copy
        cmp_file.save(args.output)
        print(f"Saved to {args.output}")


if __name__ == '__main__':
    main()
