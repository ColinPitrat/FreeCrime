import struct
import argparse
import sys
import os

# Helper to get/set by path
def get_item(obj, key):
    if isinstance(obj, dict):
        return obj.get(key)
    if isinstance(obj, list):
        try:
            return obj[int(key)]
        except (ValueError, IndexError):
            return None
    return getattr(obj, key, None)

def set_val(obj, key, val):
    if isinstance(obj, dict):
        obj[key] = val
    elif isinstance(obj, list):
        idx = int(key)
        if 0 <= idx < len(obj):
            obj[idx] = val
        else:
            raise IndexError(f"Index {idx} out of range")
    else:
        setattr(obj, key, val)

def resolve_path(root, path):
    clean_path = path.replace('[', '.').replace(']', '')
    parts = clean_path.split('.')

    current = root
    for part in parts[:-1]:
        try:
            current = get_item(current, part)
            if current is None:
                raise KeyError(f"Path segment '{part}' not found")
        except Exception as e:
            raise Exception(f"Error resolving path '{path}': {e}")

    last_part = parts[-1]
    return current, last_part

def write_bmp(filepath, width, height, pixels, palette):
    # Pad rows to 4 bytes
    row_size = width
    padding = (4 - (row_size % 4)) % 4

    # Header sizes
    file_header_size = 14
    info_header_size = 40
    palette_size = 256 * 4
    pixel_data_offset = file_header_size + info_header_size + palette_size
    file_size = pixel_data_offset + (row_size + padding) * height

    with open(filepath, 'wb') as f:
        # File Header
        f.write(b'BM')
        f.write(struct.pack('<I', file_size))
        f.write(b'\x00\x00') # Reserved
        f.write(b'\x00\x00') # Reserved
        f.write(struct.pack('<I', pixel_data_offset))

        # Info Header
        f.write(struct.pack('<I', info_header_size))
        f.write(struct.pack('<i', width))
        f.write(struct.pack('<i', -height)) # Negative height for top-down
        f.write(struct.pack('<H', 1)) # Planes
        f.write(struct.pack('<H', 8)) # Bit count
        f.write(struct.pack('<I', 0)) # Compression
        f.write(struct.pack('<I', (row_size + padding) * height)) # SizeImage
        f.write(struct.pack('<i', 0)) # XPixelsPerMeter
        f.write(struct.pack('<i', 0)) # YPixelsPerMeter
        f.write(struct.pack('<I', 256)) # ClrUsed
        f.write(struct.pack('<I', 0)) # ClrImportant

        # Palette
        # Palette input is expected to be [r, g, b, r, g, b...]
        # BMP expects B, G, R, Reserved
        for i in range(256):
            if i * 3 + 2 < len(palette):
                r = palette[i * 3]
                g = palette[i * 3 + 1]
                b = palette[i * 3 + 2]
                f.write(struct.pack('BBBB', b, g, r, 0))
            else:
                f.write(b'\x00\x00\x00\x00')

        # Pixel Data
        # If top-down (negative height), we write rows 0 to N.
        for y in range(height):
            start = y * width
            row = pixels[start : start + width]
            f.write(bytes(row))
            f.write(b'\x00' * padding)

def apply_delta(base_pixels, delta_data, sprite_width, sprite_id, delta_id):
    # Copy base to start
    pixels = list(base_pixels)
    offset = 0
    curr_x, curr_y = 0, 0

    while offset < len(delta_data):
        if offset + 3 > len(delta_data):
            print(f"WARNING: sprite {sprite_id} - delta {delta_id}: offset outside of delta: offset={offset} - len delta={len(delta_data)}", file=sys.stderr)
            break

        # Read offset (2 bytes)
        dx, dy = struct.unpack('<B B', bytes(delta_data[offset:offset+2]))
        offset += 2

        curr_x += dx
        curr_y += dy

        while curr_x >= sprite_width:
            curr_x -= 256
            curr_y += 1

        curr_pos = curr_x + curr_y*sprite_width

        # Read length (1 byte)
        length = delta_data[offset]
        offset += 1

        # Read data
        if offset + length > len(delta_data):
            print(f"WARNING: sprite {sprite_id} - delta {delta_id}: offset+length outside of delta: offset={offset}, length={length} - len delta={len(delta_data)}", file=sys.stderr)
            break
        data = delta_data[offset : offset + length]
        offset += length
        #print(f"Sprite {sprite_id} - delta {delta_id} - curr_pos={curr_pos} - dx={dx}, dy={dy}, patch={data}")

        # Apply
        # Ensure we don't go out of bounds
        end_pos = curr_pos + length
        if end_pos > len(pixels):
            print(f"WARNING: sprite {sprite_id} - delta {delta_id}: delta out of sprite bounds: end_pos={end_pos} - len sprite={len(pixels)}", file=sys.stderr)
            valid_len = len(pixels) - curr_pos
            if valid_len > 0:
                pixels[curr_pos : curr_pos + valid_len] = data[:valid_len]
        else:
            pixels[curr_pos : end_pos] = data

        curr_x += len(data)

    return pixels

class StyleFile:
    def __init__(self, filepath, verbose=False):
        self.filepath = filepath
        with open(filepath, 'rb') as f:
            self.data = f.read()
        self.parse(verbose)

    def parse(self, verbose=False):
        self.version = struct.unpack('<I', self.data[0:4])[0]
        offset = 0

        if verbose:
            print(f"Header starts at 0x{offset:08X}")

        if self.version == 336: # G24
            fmt = '<' + 'I'*16
            size = struct.calcsize(fmt)
            values = struct.unpack(fmt, self.data[offset:offset+size])
            self.header = {
                'version': values[0],
                'side_size': values[1],
                'lid_size': values[2],
                'aux_size': values[3],
                'anim_size': values[4],
                'clut_size': values[5],
                'tileclut_size': values[6],
                'spriteclut_size': values[7],
                'newcarclut_size': values[8],
                'fontclut_size': values[9],
                'palette_index_size': values[10],
                'object_info_size': values[11],
                'car_size': values[12],
                'sprite_info_size': values[13],
                'sprite_graphics_size': values[14],
                'sprite_numbers_size': values[15]
            }
        else: # GRY
            fmt = '<' + 'I'*13
            size = struct.calcsize(fmt)
            values = struct.unpack(fmt, self.data[offset:offset+size])
            self.header = {
                'version': values[0],
                'side_size': values[1],
                'lid_size': values[2],
                'aux_size': values[3],
                'anim_size': values[4],
                'palette_size': values[5],
                'remap_size': values[6],
                'remap_index_size': values[7],
                'object_info_size': values[8],
                'car_size': values[9],
                'sprite_info_size': values[10],
                'sprite_graphics_size': values[11],
                'sprite_numbers_size': values[12]
            }

        offset += size

        # Blocks Logic
        if verbose:
            print(f"Blocks start at {offset:08X}")
        num_side = self.header['side_size'] // 4096
        num_lid = self.header['lid_size'] // 4096
        num_aux = self.header['aux_size'] // 4096
        total_blocks = num_side + num_lid + num_aux

        padding_blocks = (4 - (total_blocks % 4)) % 4
        blocks_data_size = 4096*(total_blocks + padding_blocks)

        all_blocks = self.parse_raw_blocks(offset, blocks_data_size)
        offset += blocks_data_size
        self.side_block = all_blocks[:num_side]
        self.lid_block = all_blocks[num_side:num_side+num_lid]
        self.aux_block = all_blocks[num_side+num_lid:num_side+num_lid+num_aux]
        self.padding_block = all_blocks[num_side+num_lid+num_aux:]

        # Anim
        if verbose:
            print(f"Anims start at {offset:08X}")
        self.anim = self.parse_anim(offset, self.header['anim_size'])
        offset += self.header['anim_size']

        if self.version == 336: # G24
            # CLUT
            trailing_size = (self.header['palette_index_size'] +
                             self.header['object_info_size'] +
                             self.header['car_size'] +
                             self.header['sprite_info_size'] +
                             self.header['sprite_graphics_size'] +
                             self.header['sprite_numbers_size'])

            clut_end = len(self.data) - trailing_size
            clut_real_size = clut_end - offset

            self.clut_data = list(self.data[offset:offset+clut_real_size])
            offset += clut_real_size

            self.palette_index = list(self.data[offset:offset+self.header['palette_index_size']])
            offset += self.header['palette_index_size']

            self.palette = []
            self.remap_tables = []
            self.remap_index = []

        else: # GRY
            if verbose:
                print(f"Palette starts at {offset:08X}")
            self.palette = list(self.data[offset:offset+self.header['palette_size']])
            offset += self.header['palette_size']

            if verbose:
                print(f"Remap tables start at {offset:08X}")
            nb_remap_tables = self.header['remap_size'] // 256
            if nb_remap_tables * 256 != self.header['remap_size']:
                print(f"WARNING: remap_size = {self.header['remap_size']} is not a multiple of 256", file=sys.stderr)
            self.remap_tables = []
            for i in range(nb_remap_tables):
                self.remap_tables.append(list(self.data[offset+i*256:offset+(i+1)*256]))
            offset += self.header['remap_size']

            if verbose:
                print(f"Remap index starts at {offset:08X}")
            if self.header['remap_index_size'] != 1024:
                print(f"WARNING: remap_index_size = {self.header['remap_index_size']} != 1024 = 4*256", file=sys.stderr)
            self.remap_index = []
            for i in range(256):
                self.remap_index.append(list(self.data[offset+i*4:offset+(i+1)*4]))
                if self.remap_index[i][0] != 0:
                    print(f"WARNING: remap_index[{i}][0] != 0 - this is not necessarily a problem but this is a smell!", file=sys.stderr)
            offset += self.header['remap_index_size']

            self.clut_data = []
            self.palette_index = []

        if verbose:
            print(f"Objects info starts at {offset:08X}")
        self.object_info = self.parse_object_info(offset, self.header['object_info_size'])
        offset += self.header['object_info_size']

        if verbose:
            print(f"Cars info starts at {offset:08X}")
        self.car_info = self.parse_car_info(offset, self.header['car_size'])
        offset += self.header['car_size']

        # Sprites
        if verbose:
            print(f"Sprites info starts at {offset:08X}")
        gfx_offset = offset + self.header['sprite_info_size']
        if verbose:
            print(f"Sprites graphics starts at {gfx_offset:08X}")
        self.sprites_graphics_raw = self.data[gfx_offset:gfx_offset+self.header['sprite_graphics_size']]
        self.sprites = self.parse_sprites(offset, self.header['sprite_info_size'],
                                          gfx_offset, self.header['sprite_graphics_size'])

        self.sprites_pages = self.parse_sprites_pages(gfx_offset, self.header['sprite_graphics_size'])

        offset += self.header['sprite_info_size']
        offset += self.header['sprite_graphics_size']

        self.sprite_numbers = self.parse_sprite_numbers(offset, self.header['sprite_numbers_size'])
        offset += self.header['sprite_numbers_size']

        self.offsets = dict()
        self.compute_offsets()

        if verbose:
            print(f"Trailing bytes starts at {offset:08X}")
        self.remaining = self.data[offset:]

    def print_info(self):
        print(f"Version: {self.version}")
        print(f"# Total blocks: {len(self.side_block) + len(self.lid_block) + len(self.aux_block)}")
        print(f"  # Side blocks: {len(self.side_block)}")
        print(f"  # Lid blocks: {len(self.lid_block)}")
        print(f"  # Aux blocks: {len(self.aux_block)}")
        print(f"# Objects info: {len(self.object_info)}")
        print(f"# Cars info: {len(self.car_info)}")
        print(f"# Sprites: {len(self.sprites)}")
        print(f"# Sprites numbers: {len(self.sprite_numbers)}")
        print(f"# Not parsed bytes: {len(self.remaining)}")

    def parse_raw_blocks(self, offset, size):
        count = size // 4096
        blocks = [[] for _ in range(count)]
        for i in range(count//4):  # Rows of blocks (4 blocks per row)
            for j in range(64):  # 64 lines per block
                for k in range(4):  # Copy the whole line for each of the 4 sprites 
                    line_start = offset + i*4*4096 + j*256 + k*64
                    blocks[4*i+k].extend(list(self.data[line_start:line_start+64]))
        return blocks

    def pack_raw_blocks(self, blocks):
        data = bytearray()
        count = len(blocks)
        if count % 4 != 0:
            raise ValueError("Block count for packing must be a multiple of 4")
        for i in range(count//4):  # Rows of blocks (4 blocks per row)
            for j in range(64):  # 64 lines per block
                for k in range(4):  # Copy the whole line for each of the 4 sprites 
                    data.extend(bytes(blocks[4*i+k][64*j:64*(j+1)]))
        return data

    def parse_anim(self, offset, size):
        if size == 0: return {'num_anims': 0, 'anims': []}
        num_anims = self.data[offset]
        anims = []
        curr = offset + 1
        for _ in range(num_anims):
            if curr + 4 > len(self.data): break
            b, w, s, fc = struct.unpack('<B B B B', self.data[curr:curr+4])
            curr += 4
            frames = list(self.data[curr:curr+fc])
            curr += fc
            anims.append({'block': b, 'which': w, 'speed': s, 'frame_count': fc, 'frames': frames})

        parsed_size = curr - offset
        if parsed_size != size:
            print(f"WARNING: Parsed anim size {parsed_size} does not match header size {size}", file=sys.stderr)

        return {'num_anims': num_anims, 'anims': anims}

    def pack_anim(self):
        data = bytearray()
        data.append(self.anim['num_anims'])
        for a in self.anim['anims']:
            data.extend(struct.pack('<B B B B', a['block'], a['which'], a['speed'], len(a['frames'])))
            data.extend(bytes(a['frames']))
        return data

    def parse_object_info(self, offset, size):
        objects = []
        curr = offset
        end = offset + size
        while curr < end:
            #vals = struct.unpack('<i i i H H H b B', self.data[curr:curr+20])
            vals = struct.unpack('<I I I H H H B B', self.data[curr:curr+20])
            curr += 20
            num_into = vals[7]
            into = list(struct.unpack(f'<{num_into}H', self.data[curr:curr+num_into*2]))
            curr += num_into * 2
            if vals[6] > 9:
                print(f'WARNING: Unknown status {vals[6]} for object #{len(objects)}', file=sys.stderr)

            obj = {
                'width': vals[0], 'height': vals[1], 'depth': vals[2],
                'spr_num': vals[3], 'weight': vals[4], 'aux': vals[5],
                'status': vals[6], 'num_into': num_into, 'into': into
            }
            objects.append(obj)
        return objects

    def compute_offsets(self):
        keys = [
            'SPR_ARROW', 'SPR_DIGITS', 'SPR_BOAT', 'SPR_BOX', 'SPR_BUS', 'SPR_CAR', 'SPR_OBJECT',
            'SPR_PED', 'SPR_SPEEDO', 'SPR_TANK', 'SPR_TRAFFIC_LIGHTS', 'SPR_TRAIN', 'SPR_TRDOORS',
            'SPR_BIKE', 'SPR_TRAM', 'SPR_WBUS', 'SPR_WCAR', 'SPR_EX', 'SPR_TUMCAR', 'SPR_TUMTRUCK', 'SPR_FERRY'
        ]
        offset = 0
        for key in keys:
            self.offsets[key] = offset
            offset += self.sprite_numbers[key]

    def pack_object_info(self):
        data = bytearray()
        for obj in self.object_info:
            num_into = len(obj['into'])
            data.extend(struct.pack('<i i i H H H b B',
                obj['width'], obj['height'], obj['depth'], obj['spr_num'],
                obj['weight'], obj['aux'], obj['status'], num_into))
            data.extend(struct.pack(f'<{num_into}H', *obj['into']))
        return data

    def parse_car_info(self, offset, size):
        cars = []
        curr = offset
        end = offset + size
        while curr < end:
            dims = struct.unpack('<3h', self.data[curr:curr+6])
            curr += 6
            spr_num = struct.unpack('<h', self.data[curr:curr+2])[0]
            curr += 2
            weight = struct.unpack('<h', self.data[curr:curr+2])[0]
            curr += 2
            speed = struct.unpack('<2h', self.data[curr:curr+4])
            curr += 4
            acc_brake = struct.unpack('<2h', self.data[curr:curr+4])
            curr += 4
            grip_hand = struct.unpack('<2h', self.data[curr:curr+4])
            curr += 4

            remap24 = []
            for _ in range(12):
                remap24.append(list(struct.unpack('<3h', self.data[curr:curr+6])))
                curr += 6

            remap8 = list(struct.unpack('<12B', self.data[curr:curr+12]))
            curr += 12

            b_vals = struct.unpack('<4B', self.data[curr:curr+4])
            curr += 4

            value = list(struct.unpack('<4H', self.data[curr:curr+8]))
            curr += 8

            cxy = struct.unpack('<2b', self.data[curr:curr+2])
            curr += 2

            m_vals = struct.unpack('<3i', self.data[curr:curr+12])
            curr += 12

            tyre = struct.unpack('<2i', self.data[curr:curr+8])
            curr += 8

            brakes = struct.unpack('<3i', self.data[curr:curr+12])
            curr += 12

            offsets = struct.unpack('<3h', self.data[curr:curr+6])
            curr += 6

            slides = struct.unpack('<2i', self.data[curr:curr+8])
            curr += 8

            flags = struct.unpack('<6B', self.data[curr:curr+6])
            curr += 6

            doors_count = struct.unpack('<h', self.data[curr:curr+2])[0]
            curr += 2

            doors = []
            for _ in range(doors_count):
                d_vals = struct.unpack('<4h', self.data[curr:curr+8])
                curr += 8
                doors.append({'rpx': d_vals[0], 'rpy': d_vals[1], 'object': d_vals[2], 'delta': d_vals[3]})

            car = {
                'width': dims[0], 'height': dims[1], 'depth': dims[2],
                'spr_num': spr_num, 'weight': weight,
                'max_speed': speed[0], 'min_speed': speed[1],
                'acceleration': acc_brake[0], 'braking': acc_brake[1],
                'grip': grip_hand[0], 'handling': grip_hand[1],
                'remap24': remap24, 'remap8': remap8,
                'vtype': b_vals[0], 'model': b_vals[1], 'turning': b_vals[2], 'damageable': b_vals[3],
                'value': value,
                'cx': cxy[0], 'cy': cxy[1],
                'moment': m_vals[0], 'rbp_mass': m_vals[1], 'g1_thrust': m_vals[2],
                'tyre_adhesion_x': tyre[0], 'tyre_adhesion_y': tyre[1],
                'handbrake_friction': brakes[0], 'footbrake_friction': brakes[1], 'front_brake_bias': brakes[2],
                'turn_ratio': offsets[0], 'drive_wheel_offset': offsets[1], 'steering_wheel_offset': offsets[2],
                'back_end_slide_value': slides[0], 'handbrake_slide_value': slides[1],
                'convertible': flags[0], 'engine': flags[1], 'radio': flags[2],
                'horn': flags[3], 'sound_function': flags[4], 'fast_change_flag': flags[5],
                'doors': doors
            }
            cars.append(car)

        return cars

    def pack_car_info(self):
        data = bytearray()
        for car in self.car_info:
            data.extend(struct.pack('<3h', car['width'], car['height'], car['depth']))
            data.extend(struct.pack('<h', car['spr_num']))
            data.extend(struct.pack('<h', car['weight']))
            data.extend(struct.pack('<2h', car['max_speed'], car['min_speed']))
            data.extend(struct.pack('<2h', car['acceleration'], car['braking']))
            data.extend(struct.pack('<2h', car['grip'], car['handling']))

            for r in car['remap24']:
                data.extend(struct.pack('<3h', *r))

            data.extend(struct.pack('<12B', *car['remap8']))
            data.extend(struct.pack('<4B', car['vtype'], car['model'], car['turning'], car['damageable']))
            data.extend(struct.pack('<4H', *car['value']))
            data.extend(struct.pack('<2b', car['cx'], car['cy']))
            data.extend(struct.pack('<3i', car['moment'], car['rbp_mass'], car['g1_thrust']))
            data.extend(struct.pack('<2i', car['tyre_adhesion_x'], car['tyre_adhesion_y']))
            data.extend(struct.pack('<3i', car['handbrake_friction'], car['footbrake_friction'], car['front_brake_bias']))
            data.extend(struct.pack('<3h', car['turn_ratio'], car['drive_wheel_offset'], car['steering_wheel_offset']))
            data.extend(struct.pack('<2i', car['back_end_slide_value'], car['handbrake_slide_value']))
            data.extend(struct.pack('<6B', car['convertible'], car['engine'], car['radio'],
                car['horn'], car['sound_function'], car['fast_change_flag']))

            data.extend(struct.pack('<h', len(car['doors'])))
            for d in car['doors']:
                data.extend(struct.pack('<4h', d['rpx'], d['rpy'], d['object'], d['delta']))
        return data

    def extract_sprite(self, offset, width, height):
        pixels = []
        for line in range(height):
            pixels.extend(self.data[offset:offset+width])
            offset += 256
        return pixels

    def parse_sprites(self, info_offset, info_size, gfx_offset, gfx_size):
        sprites = []
        curr_info = info_offset
        end_info = info_offset + info_size

        curr_gfx = gfx_offset

        i = 0
        while curr_info < end_info:
            if self.version == 336: # G24
                # 12 bytes header
                base = struct.unpack('<B B B B H I H', self.data[curr_info:curr_info+12])
                curr_info += 12
                w, h, dc, ws, sz, ptr, extra = base
            else: # GRY
                # 10 bytes header
                base = struct.unpack('<B B B B H I', self.data[curr_info:curr_info+10])
                curr_info += 10
                w, h, dc, ws, sz, ptr = base
                extra = 0

            deltas_info = []
            for _ in range(dc):
                d = struct.unpack('<H I', self.data[curr_info:curr_info+6])
                curr_info += 6
                deltas_info.append({'size': d[0], 'ptr': d[1]})

            base_size = w * h
            if base_size != sz:
                print(f"WARNING: Sprite {i} has width={w} and height={h} but size={sz} != width*height = {base_size}", file=sys.stderr)

            if base_size > 0:
                pixels = self.extract_sprite(gfx_offset + ptr, w, h)
                curr_gfx += base_size
            else:
                pixels = []

            deltas = []
            for d_info in deltas_info:
                d_size = d_info['size']
                d_start = gfx_offset + d_info['ptr']
                if d_size > 0:
                    d_data = list(self.data[d_start:d_start+d_size])
                    curr_gfx += d_size
                else:
                    d_data = []
                deltas.append({'size': d_size, 'ptr': d_info['ptr'], 'data': d_data})

            sprite = {
                'w': w, 'h': h, 'ws': ws, 'size': sz, 'ptr': ptr, 'extra': extra,
                'pixels': pixels, 'deltas': deltas
            }
            sprites.append(sprite)
            i += 1

        if curr_info != end_info:
            print(f"WARNING: Parsed sprite info size mismatch. Expected {info_size}, Consumed {curr_info - info_offset}", file=sys.stderr)

        expected_gfx_end = gfx_offset + gfx_size
        if curr_gfx != expected_gfx_end:
             print(f"WARNING: Parsed sprite graphics size mismatch. Expected {gfx_size}, Consumed {curr_gfx - gfx_offset}", file=sys.stderr)

        self.sprite_graphics_padding = list(self.data[curr_gfx : expected_gfx_end])

        return sprites

    def parse_sprites_pages(self, offset, size):
        num_pages = size // (256*256)
        if num_pages*256*256 != size:
            print(f"WARNING: Sprite graphics area is not a integer number of pages: size={size}, pages={size/(256*256)}", file=sys.stderr)

        pages = []
        for p in range(num_pages):
            pages.append(list(self.data[offset+256*256*p:offset+256*256*(p+1)]))

        return pages

    def pack_sprites(self):
        info_data = bytearray()
        gfx_data = bytearray()

        for s in self.sprites:
            expected_base = s['w'] * s['h']
            if len(s['pixels']) != expected_base:
                if len(s['pixels']) > expected_base:
                     s['pixels'] = s['pixels'][:expected_base]
                else:
                     s['pixels'].extend([0] * (expected_base - len(s['pixels'])))

            delta_count = len(s['deltas'])

            if self.version == 336: # G24
                info_data.extend(struct.pack('<B B B B H I H', s['w'], s['h'], delta_count, s['ws'], s['w']*s['h'], s['ptr'], s.get('extra', 0)))
            else: # GRY
                info_data.extend(struct.pack('<B B B B H I', s['w'], s['h'], delta_count, s['ws'], s['w']*s['h'], s['ptr']))

            # TODO: fix overwriting gfx_data
            #gfx_data.extend(bytes(s['pixels']))

            for d in s['deltas']:
                 d_len = len(d['data'])
                 info_data.extend(struct.pack('<H I', d_len, d['ptr']))
                 #gfx_data.extend(bytes(d['data']))

        #if hasattr(self, 'sprite_graphics_padding'):
        #    gfx_data.extend(bytes(self.sprite_graphics_padding))
        gfx_data.extend(self.sprite_graphics_raw)

        return info_data, gfx_data

    def parse_sprite_numbers(self, offset, size):
        vals = struct.unpack('<' + 'H'*21, self.data[offset:offset+size])
        keys = [
            'SPR_ARROW', 'SPR_DIGITS', 'SPR_BOAT', 'SPR_BOX', 'SPR_BUS', 'SPR_CAR', 'SPR_OBJECT',
            'SPR_PED', 'SPR_SPEEDO', 'SPR_TANK', 'SPR_TRAFFIC_LIGHTS', 'SPR_TRAIN', 'SPR_TRDOORS',
            'SPR_BIKE', 'SPR_TRAM', 'SPR_WBUS', 'SPR_WCAR', 'SPR_EX', 'SPR_TUMCAR', 'SPR_TUMTRUCK', 'SPR_FERRY'
        ]
        return dict(zip(keys, vals))

    def pack_sprite_numbers(self):
        keys = [
            'SPR_ARROW', 'SPR_DIGITS', 'SPR_BOAT', 'SPR_BOX', 'SPR_BUS', 'SPR_CAR', 'SPR_OBJECT',
            'SPR_PED', 'SPR_SPEEDO', 'SPR_TANK', 'SPR_TRAFFIC_LIGHTS', 'SPR_TRAIN', 'SPR_TRDOORS',
            'SPR_BIKE', 'SPR_TRAM', 'SPR_WBUS', 'SPR_WCAR', 'SPR_EX', 'SPR_TUMCAR', 'SPR_TUMTRUCK', 'SPR_FERRY'
        ]
        vals = [self.sprite_numbers.get(k, 0) for k in keys]
        return struct.pack('<' + 'H'*21, *vals)

    def save(self, filepath):
        all_blocks_data = self.pack_raw_blocks(self.side_block + self.lid_block + self.aux_block + self.padding_block)

        anim = self.pack_anim()
        obj_info = self.pack_object_info()
        car_info = self.pack_car_info()
        spr_info, spr_gfx = self.pack_sprites()
        spr_num = self.pack_sprite_numbers()

        self.header['side_size'] = len(self.side_block)*4096
        self.header['lid_size'] = len(self.lid_block)*4096
        self.header['aux_size'] = len(self.aux_block)*4096
        self.header['anim_size'] = len(anim)
        self.header['object_info_size'] = len(obj_info)
        self.header['car_size'] = len(car_info)
        self.header['sprite_info_size'] = len(spr_info)
        self.header['sprite_graphics_size'] = len(spr_gfx)
        self.header['sprite_numbers_size'] = len(spr_num)

        with open(filepath, 'wb') as f:
            if self.version == 336: # G24
                clut_blob = bytes(self.clut_data)
                pal_idx = bytes(self.palette_index)

                self.header['palette_index_size'] = len(pal_idx)

                f.write(struct.pack('<' + 'I'*16,
                    self.header['version'], self.header['side_size'], self.header['lid_size'], self.header['aux_size'],
                    self.header['anim_size'], self.header['clut_size'], self.header['tileclut_size'], self.header['spriteclut_size'],
                    self.header['newcarclut_size'], self.header['fontclut_size'],
                    self.header['palette_index_size'],
                    self.header['object_info_size'], self.header['car_size'], self.header['sprite_info_size'],
                    self.header['sprite_graphics_size'], self.header['sprite_numbers_size']))

                f.write(all_blocks_data)
                f.write(anim)
                f.write(clut_blob)
                f.write(pal_idx)

            else: # GRY
                pal = bytes(self.palette)
                remap = bytes(self.remap_tables)
                remap_idx = bytes(self.remap_index)

                self.header['palette_size'] = len(pal)
                self.header['remap_size'] = len(remap)
                self.header['remap_index_size'] = len(remap_idx)

                f.write(struct.pack('<' + 'I'*13,
                    self.header['version'], self.header['side_size'], self.header['lid_size'], self.header['aux_size'],
                    self.header['anim_size'], self.header['palette_size'], self.header['remap_size'], self.header['remap_index_size'],
                    self.header['object_info_size'], self.header['car_size'], self.header['sprite_info_size'],
                    self.header['sprite_graphics_size'], self.header['sprite_numbers_size']))

                f.write(all_blocks_data)
                f.write(anim)
                f.write(pal)
                f.write(remap)
                f.write(remap_idx)

            f.write(obj_info)
            f.write(car_info)
            f.write(spr_info)
            f.write(spr_gfx)
            f.write(spr_num)
            f.write(self.remaining)

    def vehicle_type_html(self, vtype):
        if vtype == 0:
            return "bus"
        if vtype == 1:
            return "juggernaut (front)"
        if vtype == 2:
            return "juggernaut (back)"
        if vtype == 3:
            return "motorcycle"
        if vtype == 4:
            return "car"
        if vtype == 8:
            return "train"
        if vtype == 9:
            return "tram"
        if vtype == 13:
            return "boat"
        if vtype == 14:
            return "tank"
        return f"<b>unknown vehicle ({vtype})</b>"

    def vehicle_type_const(self, vtype):
        if vtype == 0:
            return "SPR_BUS"
        #if vtype == 1:
        #    return "front of juggernaut"
        #if vtype == 2:
        #    return "back of juggernaut"
        if vtype == 3:
            return "SPR_BIKE"
        if vtype == 4:
            return "SPR_CAR"
        if vtype == 8:
            return "SPR_TRAIN"
        if vtype == 9:
            return "SPR_TRAM"
        if vtype == 13:
            return "SPR_BOAT"
        if vtype == 14:
            return "SPR_TANK"
        return "SPR_CAR"

    def write_css(self, out_dir, filename):
        with open(os.path.join(out_dir, filename), 'w') as f:
            f.write('body {\n  background-color: #A0A0A0;\n}\n')
            f.write('div.object {\n  width: 100%;\n  margin: 0\n  padding: 0\n  display: flex;\n}\n\n');
            f.write('span.infos {\n  width: 49%;\n  display: inline-block;\n}\n\n');
            f.write('th {\n  text-align: left;\n}\n');
            f.write('td {\n  padding-right: 2em;\n}\n');
            f.write('span.sprites {\n  width: 49%;\n  display: inline-block;\n}\n\n');
            f.write('span.sprites img {\n  margin-right: 0.5em;\n}\n\n');
            #f.write('span.sprites img:hover {\n  transform: scale(10);\n}\n\n');

    def status_html(self, status_id):
        if status_id == 0:
            return 'normal'
        if status_id == 1:
            return 'ignorable'
        if status_id == 2:
            return 'smashable'
        if status_id == 3:
            return 'invisible'
        if status_id == 5:
            return 'animation'
        if status_id == 6:
            return 'weapon?'
        if status_id == 9:
            return 'bonus'
        return f"<b>invalid ({status_id})</b>";

    def write_html(self, out_dir, filename):
        html_filename = filename + '.html'
        title = f'Objects and cars from {filename}'
        with open(os.path.join(out_dir, html_filename), 'w') as f:
            f.write(f'<!DOCTYPE html>\n<html>\n<head>\n<title>{title}</title>\n<link rel="stylesheet" href="styles.css">\n</head>\n<body>\n<h1>{title}</h1>\n')

            f.write(f'<h2>Objects</h2>\n')
            for o, obj in enumerate(self.object_info):
                f.write(f'<h3>Object #{o}</h3>\n')
                f.write(f'<div class="object"><span class="infos"><table>\n')
                for name, prop in { 'Width': 'width', 'Height': 'height', 'Depth': 'depth', 'Weight': 'weight' }.items():
                    f.write(f'<tr><th>{name}: </th><td>{obj[prop]}</td></tr>\n')
                status = self.status_html(obj['status'])
                f.write(f'<tr><th>Status: </th><td>{status}</td></tr>\n')
                f.write(f'</table></span>\n')
                f.write(f'<span class="sprites">')
                if obj['status'] != 3:  # no sprite for invisible objects
                    nb_sprites = 1
                    if obj['status'] == 5:  # Animated objects: number of sprites is in width
                        nb_sprites = obj['width']
                    if obj['status'] == 9:  # Bonus: always 8 sprites
                        nb_sprites = 8
                    for i in range(nb_sprites):
                        f.write(f'<a href="sprite_{obj['spr_num']+self.offsets['SPR_OBJECT']+i:03}.bmp"><img src="sprite_{obj['spr_num']+self.offsets['SPR_OBJECT']+i:03}.bmp" style="height: 100px;"/></a>')
                f.write(f'</span></div>')
                f.write(f'<hr/>')

            f.write(f'<h2>Vehicles</h2>\n')
            for c, car in enumerate(self.car_info):
                f.write(f'<h3>Vehicle #{c}</h3>\n')
                f.write(f'<div class="object"><span class="infos"><table>\n')
                f.write(f'<tr>\n')
                columns = 4
                i = 0
                for name, prop in {
                    'Width': 'width', 'Height': 'height', 'Depth': 'depth', 'Weight': 'weight',
                    'Type': 'vtype', 'Model': 'model',
                    'Minimum speed': 'min_speed', 'Maximum speed': 'max_speed',
                    'Acceleration': 'acceleration', 'Braking': 'braking',
                    'Grip': 'grip', 'Handling': 'handling', 'Turning': 'turning',
                    'Damageable': 'damageable', 'Convertible': 'convertible',
                    'Engine': 'engine', 'Radio': 'radio',
                    'Turn ratio': 'turn_ratio', 'Drive wheel offset': 'drive_wheel_offset', 'Steering wheel offset': 'steering_wheel_offset',
                    'Back-end slide': 'back_end_slide_value', 'Handbrake slide': 'handbrake_slide_value',
                    'Handbrake friction': 'handbrake_friction', 'Footbrake friction': 'footbrake_friction', 'Front brake bias': 'front_brake_bias',
                    'Horn': 'horn', 'Sound function': 'sound_function', 'Fast change flag': 'fast_change_flag',
                }.items():
                    value = car[prop]
                    if prop == 'vtype':
                        value = self.vehicle_type_html(car[prop])
                    f.write(f'<th>{name}: </th><td>{value}</td>')
                    i+=1
                    if i % columns == 0:
                        f.write(f'</tr>\n<tr>')
                f.write(f'</tr>\n')
                f.write(f'</table></span>\n')
                f.write(f'<span class="sprites">')
                sprite_num = car['spr_num']+self.offsets[self.vehicle_type_const(car['vtype'])]
                f.write(f'<a href="sprite_{sprite_num:03}.bmp"><img src="sprite_{sprite_num:03}.bmp" style="height: 200px;"/></a>')
                f.write(f'</span></div>')
                f.write(f'<hr/>')
            # TODO
            if False:
                    for r in car['remap24']:
                        data.extend(struct.pack('<3h', *r))

                    data.extend(struct.pack('<12B', *car['remap8']))
                    data.extend(struct.pack('<4H', *car['value']))
                    data.extend(struct.pack('<2b', car['cx'], car['cy']))
                    data.extend(struct.pack('<3i', car['moment'], car['rbp_mass'], car['g1_thrust']))
                    data.extend(struct.pack('<2i', car['tyre_adhesion_x'], car['tyre_adhesion_y']))

                    data.extend(struct.pack('<h', len(car['doors'])))
                    for d in car['doors']:
                        data.extend(struct.pack('<4h', d['rpx'], d['rpy'], d['object'], d['delta']))

            f.write('</body>\n</html>\n')

def main():
    parser = argparse.ArgumentParser(description='Modify Style files (GRY/G24)')
    parser.add_argument('input_file', help='Input file')
    parser.add_argument('--output', '-o', help='Output file (default: overwrite input)')
    parser.add_argument('--set', '-s', action='append', help='Set field value, e.g. object_info[0].width=20')
    parser.add_argument('--print', '-p', action='append', help='Print field value, e.g. header.version')
    parser.add_argument('--info', '-i', action='store_true', help='Print high level info about the result of the parsing')
    parser.add_argument('--export', '-e', help='Export pictures to directory')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')

    args = parser.parse_args()

    style_file = StyleFile(args.input_file, args.verbose)

    if args.info:
        style_file.print_info()

    if args.export:
        out_dir = args.export
        if not os.path.exists(out_dir):
            os.makedirs(out_dir)

        # Determine Palette
        if style_file.version == 336: # G24
            # Try to get from clut_data
            if hasattr(style_file, 'clut_data') and len(style_file.clut_data) >= 768:
                palette = style_file.clut_data[:768]
            else:
                palette = [i for i in range(256) for _ in range(3)] # Grayscale fallback
        else:
            palette = style_file.palette

        # Export Blocks
        block_types = [
            ('side_blocks', style_file.side_block),
            ('lid_blocks', style_file.lid_block),
            ('aux_blocks', style_file.aux_block)
        ]

        for name, blocks in block_types:
            for i, blk in enumerate(blocks):
                fname = f"{name}_{i:03d}.bmp"
                fpath = os.path.join(out_dir, fname)
                write_bmp(fpath, 64, 64, blk, palette)

        # Export Sprites
        if isinstance(style_file.sprites, list) and len(style_file.sprites) > 0 and 'error' not in style_file.sprites[0]:
            for i, spr in enumerate(style_file.sprites):
                w, h = spr['w'], spr['h']
                if w == 0 or h == 0: continue

                fname = f"sprite_{i:03d}.bmp"
                fpath = os.path.join(out_dir, fname)
                write_bmp(fpath, w, h, spr['pixels'], palette)

                # Deltas
                for j, delta in enumerate(spr.get('deltas', [])):
                    d_pixels = apply_delta(spr['pixels'], delta['data'], w, i, j)
                    dname = f"sprite_{i:03d}_delta_{j:03d}.bmp"
                    dpath = os.path.join(out_dir, dname)
                    write_bmp(dpath, w, h, d_pixels, palette)

        # Export Sprites pages
        for i, page in enumerate(style_file.sprites_pages):
                fname = f"sprites_page_{i:03d}.bmp"
                fpath = os.path.join(out_dir, fname)
                write_bmp(fpath, 256, 256, page, palette)

        # Export HTML page with objects & cars info
        style_file.write_css(out_dir, 'styles.css')
        style_file.write_html(out_dir, args.input_file)

        print(f"Exported images to {out_dir}")

    if args.print:
        for p in args.print:
            try:
                parent, key = resolve_path(style_file, p)
                val = get_item(parent, key)
                print(f"{p} = {val}")
            except Exception as e:
                print(f"Error reading {p}: {e}")

    if args.set:
        for s in args.set:
            if '=' not in s:
                print(f"Invalid set format: {s}. Expected field=value")
                continue
            path, value = s.split('=', 1)

            # Check prohibited paths: sprites graphics are not yet modifiable.
            # sprites info is technically so we could be more relaxed.
            if path.startswith('sprites'):
                print(f"Error: Modifying {path} is not supported yet.")
                sys.exit(1)

            try:
                parent, key = resolve_path(style_file, path)

                current_val = get_item(parent, key)
                if current_val is not None:
                    target_type = type(current_val)
                    if target_type == int:
                        new_val = int(value)
                    elif target_type == float:
                        new_val = float(value)
                    elif target_type == str:
                        new_val = value
                    elif target_type == list:
                        if value.startswith('[') and value.endswith(']'):
                             items = value[1:-1].split(',')
                             if all(x.strip().isdigit() for x in items):
                                 new_val = [int(x) for x in items]
                             else:
                                 print(f"Cannot set list from string unless integers: {value}")
                                 continue
                        else:
                             print(f"Cannot set list type directly without array syntax")
                             continue
                    else:
                        new_val = value
                else:
                    if value.isdigit():
                        new_val = int(value)
                    else:
                        try:
                            new_val = float(value)
                        except:
                            new_val = value

                set_val(parent, key, new_val)
                print(f"Set {path} to {new_val}")
            except Exception as e:
                print(f"Error setting {path}: {e}")
                sys.exit(1)

        output_path = args.output if args.output else args.input_file
        style_file.save(output_path)
        print(f"Saved to {output_path}")

if __name__ == '__main__':
    main()
