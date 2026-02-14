import argparse
import cv2
import math
import numpy as np
import pygame
from pygame import gfxdraw
import struct
import sys
import os

class G24Parser:
    def __init__(self, filepath):
        with open(filepath, 'rb') as f:
            self.data = f.read()
        self.parse_header()
        self.parse_blocks()
        if self.version == 336: # G24
            self.parse_clut()
            self.parse_pal_index()
        else: # GRY
            self.parse_palette_and_remaps()
        self.parse_object_info()
        self.parse_car_info()
        self.parse_sprite_info()
        self.parse_sprite_graphics()

    def parse_header(self):
        self.version = struct.unpack('<I', self.data[0:4])[0]

        if self.version == 336: # G24
            fmt = '<' + 'I' * 16
            size = struct.calcsize(fmt)
            values = struct.unpack(fmt, self.data[:size])
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
            values = struct.unpack(fmt, self.data[:size])
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
        self.offset = size

    def parse_palette_and_remaps(self):
        self.palette = self.data[self.offset : self.offset + self.header['palette_size']]
        self.offset += self.header['palette_size']
        self.remap_tables = self.data[self.offset : self.offset + self.header['remap_size']]
        self.offset += self.header['remap_size']
        self.remap_index = self.data[self.offset : self.offset + self.header['remap_index_size']]
        self.offset += self.header['remap_index_size']

    def deinterleave_blocks(self, data, count):
        blocks = [[0]*4096 for _ in range(count)]
        rows = count // 4
        for r in range(rows):
            for line in range(64):
                base_idx = (r * 64 + line) * 256
                row_data = data[base_idx : base_idx + 256]
                for b in range(4):
                    block_idx = r * 4 + b
                    line_data = row_data[b*64 : (b+1)*64]
                    start = line * 64
                    blocks[block_idx][start : start + 64] = line_data
        return blocks

    def parse_blocks(self):
        num_side = self.header['side_size'] // 4096
        num_lid = self.header['lid_size'] // 4096
        num_aux = self.header['aux_size'] // 4096
        total_blocks = num_side + num_lid + num_aux
        padding_blocks = (4 - (total_blocks % 4)) % 4
        total_slots = total_blocks + padding_blocks

        size = total_slots * 4096
        raw_blocks_data = self.data[self.offset : self.offset + size]
        self.offset += size

        all_blocks = self.deinterleave_blocks(raw_blocks_data, total_slots)
        self.side_blocks = all_blocks[:num_side]
        self.lid_blocks = all_blocks[num_side : num_side + num_lid]
        self.aux_blocks = all_blocks[num_side + num_lid : num_side + num_lid + num_aux]

        self.parse_anim()

    def parse_anim(self):
        end = self.offset + self.header['anim_size']
        self.animations = []
        if self.header['anim_size'] > 0:
            num_anims = self.data[self.offset]
            curr = self.offset + 1
            for _ in range(num_anims):
                block, which, speed, frame_count = struct.unpack('<BBBB', self.data[curr:curr+4])
                curr += 4
                frames = list(struct.unpack('<' + 'B' * frame_count, self.data[curr:curr+frame_count]))
                curr += frame_count
                self.animations.append({
                    'block': block, 'which': which, 'speed': speed,
                    'frame_count': frame_count, 'frames': frames
                })
        self.offset = end

    def parse_clut(self):
        paged_clut_size = self.header['clut_size']
        if paged_clut_size % 65536 != 0:
            paged_clut_size += (65536 - (paged_clut_size % 65536))
        self.clut_data = self.data[self.offset : self.offset + paged_clut_size]
        self.offset += paged_clut_size

    def parse_pal_index(self):
        size = self.header['palette_index_size']
        fmt = '<' + 'H' * (size // 2)
        self.pal_index = struct.unpack(fmt, self.data[self.offset : self.offset + size])
        self.offset += size

    def parse_object_info(self):
        size = self.header['object_info_size']
        self.object_info = []
        curr = self.offset
        end = self.offset + size
        while curr < end:
            vals = struct.unpack('<i i i H H H b B', self.data[curr:curr+20])
            curr += 20
            num_into = vals[7]
            curr += num_into * 2
            self.object_info.append({
                'width': vals[0], 'height': vals[1], 'depth': vals[2],
                'spr_num': vals[3], 'status': vals[6],
            })
        self.offset = end

    def parse_car_info(self):
        self.car_info = []
        curr = self.offset
        end = self.offset + self.header['car_size']
        while curr < end:
            vals = struct.unpack_from('<hhhh', self.data, curr)
            vtype = self.data[curr + 106]
            model = self.data[curr + 107]
            doors_count = struct.unpack_from('<h', self.data, curr + 172)[0]
            curr += 174 + doors_count * 8
            self.car_info.append({
                'width': vals[0], 'height': vals[1], 'depth': vals[2],
                'spr_num': vals[3], 'vtype': vtype, 'model': model,
            })
        self.offset = end

    def parse_sprite_info(self):
        size = self.header['sprite_info_size']
        self.sprite_info = []
        curr = self.offset
        end = self.offset + size
        while curr < end:
            if self.version == 336: # G24
                w, h, dc, v = struct.unpack('<B B B B', self.data[curr:curr+4])
                sz, clut, xoff, yoff, page = struct.unpack('<H H B B H', self.data[curr+4:curr+12])
                curr += 12
                for _ in range(dc):
                    curr += 6
                self.sprite_info.append({
                    'w': w, 'h': h, 'clut': clut, 'page': page,
                    'xoff': xoff, 'yoff': yoff, 'dc': dc
                })
            else: # GRY
                w, h, dc, v = struct.unpack('<B B B B', self.data[curr:curr+4])
                sz, ptr = struct.unpack('<H I', self.data[curr+4:curr+10])
                curr += 10
                for _ in range(dc):
                    curr += 6
                self.sprite_info.append({
                    'w': w, 'h': h, 'dc': dc, 'ptr': ptr
                })
        self.offset = end

    def parse_sprite_graphics(self):
        size = self.header['sprite_graphics_size']
        self.sprite_graphics = self.data[self.offset : self.offset + size]
        self.offset += size
        self.parse_sprite_numbers()

    def parse_sprite_numbers(self):
        size = self.header['sprite_numbers_size']
        if size == 0:
            self.sprite_bases = {}
            return
        fmt = '<' + 'H' * (size // 2)
        vals = struct.unpack(fmt, self.data[self.offset : self.offset + size])
        self.offset += size
        names = [
            'arrow', 'digits', 'boat', 'box', 'bus', 'car', 'object', 'ped',
            'speedo', 'tank', 'traffic_lights', 'train', 'trdoors', 'bike',
            'tram', 'wbus', 'wcar', 'ex', 'tumcar', 'tumtruck', 'ferry'
        ]
        self.sprite_bases = {}
        current_base = 0
        for i, name in enumerate(names):
            if i < len(vals):
                self.sprite_bases[name] = current_base
                current_base += vals[i]

    def get_color(self, clut_idx, color_idx):
        if self.version == 336: # G24
            page = clut_idx // 64
            sub = clut_idx % 64
            off = page * 65536 + color_idx * 256 + sub * 4
            if off + 3 > len(self.clut_data):
                return (0, 0, 0, 0)
            b = self.clut_data[off]
            g = self.clut_data[off+1]
            r = self.clut_data[off+2]
            a = 255 if color_idx != 0 else 0
            return (r, g, b, a)
        else: # GRY
            # clut_idx is the remap table index
            if clut_idx*256 + color_idx >= len(self.remap_tables):
                return (0, 0, 0, 0)
            palette_idx = self.remap_tables[clut_idx * 256 + color_idx]
            off = palette_idx * 3
            r = self.palette[off] * 4
            g = self.palette[off+1] * 4
            b = self.palette[off+2] * 4
            a = 255 if color_idx != 0 else 0
            return (r, g, b, a)

    def get_palette(self, clut_idx):
        pal = []
        for i in range(256):
            pal.append(self.get_color(clut_idx, i))
        return pal

class CMPParser:
    def __init__(self, filepath):
        with open(filepath, 'rb') as f:
            self.data = f.read()
        self.parse()

    def parse(self):
        offset = 0
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
        base_size = 256 * 256 * 4
        self.base = struct.unpack('<' + 'I' * (256 * 256), self.data[offset:offset+base_size])
        offset += base_size
        self.column_data = self.data[offset : offset + self.header['column_size']]
        offset += self.header['column_size']
        self.block_data = self.data[offset : offset + self.header['block_size']]
        offset += self.header['block_size']
        obj_size = self.header['object_pos_size']
        self.objects = []
        fmt = '<H H H B B H H H'
        stride = struct.calcsize(fmt)
        for i in range(obj_size // stride):
            vals = struct.unpack_from(fmt, self.data, offset + i * stride)
            self.objects.append({
                'x': vals[0], 'y': vals[1], 'z': vals[2],
                'type': vals[3], 'remap': vals[4],
                'rotation': vals[5]
            })
        self.objects = sorted(self.objects, key=lambda obj: -obj['z'])
        offset += obj_size
        offset += self.header['route_size']
        offset += 108 # location_data (3x6x6)
        nav_size = self.header['nav_data_size']
        self.nav_data = []
        stride = 35
        for i in range(nav_size // stride):
            vals = struct.unpack_from('<BBBBB30s', self.data, offset + i * stride)
            name = vals[5].split(b'\x00')[0].decode('ascii', errors='replace')
            self.nav_data.append({
                'x': vals[0], 'y': vals[1], 'w': vals[2], 'h': vals[3],
                'sam': vals[4], 'name': name
            })

    def get_column(self, x, y):
        # Swap x and y in base indexing? Let's try row-major y*256+x first, then x*256+y if needed
        base_idx = (y % 256) * 256 + (x % 256)
        col_offset = self.base[base_idx]
        if col_offset >= len(self.column_data):
            return 6, []
        height = struct.unpack_from('<H', self.column_data, col_offset)[0]
        num_blocks = 6 - height
        blocks = []
        for i in range(num_blocks):
            blk_idx = struct.unpack_from('<H', self.column_data, col_offset + 2 + i * 2)[0]
            blocks.append(self.get_block(blk_idx))
        return height, blocks

    def get_block(self, idx):
        fmt = '<H B B B B B B'
        stride = 8
        vals = struct.unpack_from(fmt, self.block_data, idx * stride)
        typemap = vals[0]
        typemapext = vals[1]
        directions = { 'up': typemap & 0x01, 'down': typemap & 0x02, 'left': typemap & 0x04, 'right': typemap & 0x08 }
        blocktype = (typemap & 0x70) >> 4
        flat = (typemap & 0x80) >> 7
        slope = (typemap & 0x3F00) >> 8
        lid_rotation = (typemap & 0xC000) >> 14
        traffic_lights = typemapext & 0x7
        lid_remap = (typemapext & 0x18) >> 3
        flip_top_bottom = (typemapext & 0x20) >> 5
        flip_left_right = (typemapext & 0x40) >> 6
        railway = (typemapext & 0x80) >> 7
        return {
            #'type_map': typemap, 'type_map_ext': typemapext,
            'left': vals[2], 'right': vals[3], 'top': vals[4], 'bottom': vals[5], 'lid': vals[6],
            'directions': directions, 'blocktype': blocktype, 'flat': flat, 'slope': slope, 'lid_rotation': lid_rotation,
            'traffic_lights': traffic_lights, 'lid_remap': lid_remap, 'flip_top_bottom': flip_top_bottom, 'flip_left_right': flip_left_right, 'railway': railway,
        }

# A map of slope type to delta for top-left, top-right, bottom-left and bottom-right corners in blocks.
slope_to_delta = {
    # 0: no slope
    0: (0, 0, 0, 0),

    # 1-8: 26° slope
    # 1-2: north
    1: (0.5, 0.5, 1, 1),
    2: (0, 0, 0.5, 0.5),
    # 3-4: south
    3: (1, 1, 0.5, 0.5),
    4: (0.5, 0.5, 0, 0),
    # 5-6: west
    5: (0.5, 1, 0.5, 1),
    6: (0, 0.5, 0, 0.5),
    # 7-8: east
    7: (1, 0.5, 1, 0.5),
    8: (0.5, 0, 0.5, 0),

    # 9-40: 7° slope
    # 9-16: north
    9: (0.875, 0.875, 1, 1),
    10: (0.75, 0.75, 0.875, 0.875),
    11: (0.625, 0.625, 0.75, 0.75),
    12: (0.5, 0.5, 0.625, 0.625),
    13: (0.375, 0.375, 0.5, 0.5),
    14: (0.25, 0.25, 0.375, 0.375),
    15: (0.125, 0.125, 0.25, 0.25),
    16: (0, 0, 0.125, 0.125),
    # 17-24: south
    17: (1, 1, 0.875, 0.875),
    18: (0.875, 0.875, 0.75, 0.75),
    19: (0.75, 0.75, 0.625, 0.625),
    20: (0.625, 0.625, 0.5, 0.5),
    21: (0.5, 0.5, 0.375, 0.375),
    22: (0.375, 0.375, 0.25, 0.25),
    23: (0.25, 0.25, 0.125, 0.125),
    24: (0.125, 0.125, 0, 0),
    # 25-32: west
    25: (0.875, 1, 0.875, 1),
    26: (0.75, 0.875, 0.75, 0.875),
    27: (0.625, 0.75, 0.625, 0.75),
    28: (0.5, 0.625, 0.5, 0.625),
    29: (0.375, 0.5, 0.375, 0.5),
    30: (0.25, 0.375, 0.25, 0.375),
    31: (0.125, 0.25, 0.125, 0.25),
    32: (0, 0.125, 0, 0.125),
    # 33-40: east
    33: (1, 0.875, 1, 0.875),
    34: (0.875, 0.75, 0.875, 0.75),
    35: (0.75, 0.625, 0.75, 0.625),
    36: (0.625, 0.5, 0.625, 0.5),
    37: (0.5, 0.375, 0.5, 0.375),
    38: (0.375, 0.25, 0.375, 0.25),
    39: (0.25, 0.125, 0.25, 0.125),
    40: (0.125, 0, 0.125, 0),

    # 41-44: 45° slope
    # 41: north
    41: (0, 0, 1, 1),
    # 42: south
    42: (1, 1, 0, 0),
    # 43: west
    43: (0, 1, 0, 1),
    # 43: east
    44: (1, 0, 1, 0),
}

class MapRenderer:
    def __init__(self, cmp_file, g24_file, show_objects=True, show_tiles=True, show_sides=True, show_lids=True, min_z=0, max_z=6, width=1024, height=768, fullscreen=False):
        self.cmp = CMPParser(cmp_file)
        self.g24 = G24Parser(g24_file)
        self.show_objects = show_objects
        self.show_tiles = show_tiles
        self.show_sides = show_sides
        self.show_lids = show_lids
        self.min_z, self.max_z = min_z, max_z
        self.screen_width, self.screen_height = width, height
        self.view_x, self.view_y = -4.0, -4.0
        #self.view_x, self.view_y = 128.0, 128.0
        #self.view_x, self.view_y = 155.0, 144.0
        #self.view_x, self.view_y = 70.0, 80.0
        self.base_tile_size = 64
        self.base_scale = 4.0
        self.clicked_x, self.clicked_y = 0, 0
        self.display_tiles_h = int(self.screen_width / 64 + 1)  # Number of tiles to display horizontally
        self.display_tiles_v = int(self.screen_height / 64 + 1)  # Number of tiles to display vertically
        self.scale_factor = 0.1
        self.surface_cache = {}
        self.sprite_cache = {}
        self.fullscreen = fullscreen
        # Pre-generate a grid of A, B coordinates (0 to 63)
        # This avoids re-creating the coordinate space every call
        self.A_grid, self.B_grid = np.meshgrid(np.arange(64), np.arange(64), indexing='ij')
        self.AB_grid = self.A_grid * self.B_grid
        pygame.init()
        self.font = pygame.font.SysFont('monospace', 24, bold=True)
        self.clock = pygame.time.Clock()
        self.init_display()
        self.apply_remaps = True
        self.play_mode = False

    def init_display(self):
        if self.fullscreen:
            self.screen = pygame.display.set_mode((self.screen_width, self.screen_height), pygame.FULLSCREEN | pygame.SCALED)
        else:
            self.screen = pygame.display.set_mode((self.screen_width, self.screen_height))

    def get_animated_block(self, block_idx, which, ticks):
        for anim in self.g24.animations:
            if anim['block'] == block_idx and anim['which'] == which:
                total_frames = anim['frame_count']# + 1
                # 1 game cycle == 1/20th of a second
                frame_idx = (ticks // (max(1, anim['speed']) * 1000 // 20)) % total_frames
                if frame_idx == 0:
                    return False, block_idx
                else:
                    return True, anim['frames'][frame_idx - 1]
        return False, block_idx

    def get_tile_surface(self, type_name, idx, ticks=0, remap=0):
        which = 1 if type_name == 'lid' else 0
        aux, idx = self.get_animated_block(idx, which, ticks)
        if aux:
            type_name = 'aux'
        key = (type_name, idx, remap)
        if key in self.surface_cache: return self.surface_cache[key]
        if idx == 0: return None
        num_side = len(self.g24.side_blocks)
        num_lid = len(self.g24.lid_blocks)
        if self.g24.version == 336: # G24
            if type_name == 'side':
                if idx >= len(self.g24.side_blocks): return None
                pixels = self.g24.side_blocks[idx]
                clut_idx = self.g24.pal_index[4*idx]
            elif type_name == 'lid':
                if idx >= len(self.g24.lid_blocks): return None
                pixels = self.g24.lid_blocks[idx]
                clut_idx = self.g24.pal_index[4 * (idx + num_side) + remap]
            elif type_name == 'aux':
                if idx >= len(self.g24.aux_blocks): return None
                pixels = self.g24.aux_blocks[idx]
                clut_idx = self.g24.pal_index[4 * (idx + num_side + num_lid)]
            else: return None
        else: # GRY
            if type_name == 'side':
                if idx >= len(self.g24.side_blocks): return None
                pixels = self.g24.side_blocks[idx]
                clut_idx = 0
            elif type_name == 'lid':
                if idx >= len(self.g24.lid_blocks): return None
                pixels = self.g24.lid_blocks[idx]
                clut_idx = self.g24.remap_index[idx * 4 + remap]
            elif type_name == 'aux':
                if idx >= len(self.g24.aux_blocks): return None
                pixels = self.g24.aux_blocks[idx]
                clut_idx = 0
            else: return None

        palette = self.g24.get_palette(clut_idx)
        surf = pygame.Surface((64, 64), pygame.SRCALPHA)
        for y in range(64):
            for x in range(64):
                surf.set_at((x, y), palette[pixels[y*64 + x]])
        self.surface_cache[key] = surf
        return surf

    def get_sprite_surface(self, spr_num, remap):
        key = (spr_num, remap)
        if key in self.sprite_cache: return self.sprite_cache[key]
        if spr_num >= len(self.g24.sprite_info): return None
        info = self.g24.sprite_info[spr_num]
        w, h = info['w'], info['h']
        if w == 0 or h == 0: return None

        if self.g24.version == 336: # G24
            page_size = 256 * 256
            pixel_start = info['page'] * page_size + info['yoff'] * 256 + info['xoff']
            tile_clut_count = self.g24.header['tileclut_size'] // 1024
            sprite_clut_count = self.g24.header['spriteclut_size'] // 1024

            # Virtual palette index
            virtual_clut = tile_clut_count + info['clut']
            if remap > 0:
                virtual_clut = tile_clut_count + sprite_clut_count + remap

            if virtual_clut >= len(self.g24.pal_index):
                clut_idx = 0
            else:
                clut_idx = self.g24.pal_index[virtual_clut]
            stride = 256
        else: # GRY
            pixel_start = info['ptr']
            clut_idx = 0
            if remap > 0:
                clut_idx = remap
            stride = 256

        palette = self.g24.get_palette(clut_idx)
        surf = pygame.Surface((w, h), pygame.SRCALPHA)
        for py in range(h):
            row_start = pixel_start + py * stride
            for px in range(w):
                if row_start + px < len(self.g24.sprite_graphics):
                    surf.set_at((px, py), palette[self.g24.sprite_graphics[row_start + px]])
        self.sprite_cache[key] = surf
        return surf

    def world_to_screen(self, x, y, z):
        h = 5 - z
        scale = self.base_scale * (1.0 + h * self.scale_factor * self.base_scale)
        sx = (x - self.view_x - self.display_tiles_h // 2) * scale * self.base_tile_size + self.screen_width // 2
        sy = (y - self.view_y - self.display_tiles_v // 2) * scale * self.base_tile_size + self.screen_height // 2
        return sx, sy, scale

    def screen_to_world(self, sx, sy, z):
        h = 5 - z
        scale = self.base_scale * (1.0 + h * self.scale_factor * self.base_scale)
        x = (sx - self.screen_width // 2) / scale / self.base_tile_size + self.view_x + self.display_tiles_h // 2
        y = (sy - self.screen_height // 2) / scale / self.base_tile_size + self.view_y + self.display_tiles_v // 2
        return x, y

    # Slow version
    def draw_textured_side_slow(self, surf, p1, p2, p3, p4):
        if not surf: return
        # We want to map the 64x64 texture onto a trapezoid p1, p2, p3, p4.
        # We apply the transformation:
        # (X,Y) = (a*A + b*B + c*A*B + d, e*A + f*B + g*A*B + h)
        # where (A,B) are the coordinates in the sprite and (X,Y) on the screen.
        d = p2[0]
        h = p2[1]
        a = (p1[0]-p2[0])/64
        e = (p1[1]-p2[1])/64
        b = (p3[0]-p2[0])/64
        f = (p3[1]-p2[1])/64
        c = (p4[0]+p2[0]-p1[0]-p3[0])/4096
        g = (p4[1]+p2[1]-p1[1]-p3[1])/4096
        steps = 64
        for A in range(steps):
            for B in range(steps):
                color = surf.get_at((A, B))
                X = int(a*A + b*B + c*A*B + d)
                Y = int(e*A + f*B + g*A*B + h)
                gfxdraw.pixel(self.screen, X, Y, color)
        if False:
            for i in range(steps):
                t1, t2 = i / float(steps), (i + 1) / float(steps)
                st_l = (p1[0]*(1-t1) + p4[0]*t1, p1[1]*(1-t1) + p4[1]*t1)
                st_r = (p2[0]*(1-t1) + p3[0]*t1, p2[1]*(1-t1) + p3[1]*t1)
                sb_l = (p1[0]*(1-t2) + p4[0]*t2, p1[1]*(1-t2) + p4[1]*t2)
                sb_r = (p2[0]*(1-t2) + p3[0]*t2, p2[1]*(1-t2) + p3[1]*t2)
                #color = surf.get_at((32, min(63, int(t1*64))))
                color = surf.get_at((min(63, int(t2*64)), min(63, int(t1*64))))
                pygame.draw.polygon(self.screen, color, [st_l, st_r, sb_r, sb_l])

    def draw_textured_side_better(self, surf, p1, p2, p3, p4):
        if not surf: return

        # 1. Grab texture colors
        # array3d returns [width, height, RGB]
        texture_array = pygame.surfarray.array3d(surf)

        # 2. Coefficients
        d, h = (p2[0], p2[1])
        a = (p1[0] - p2[0]) / 64.0
        e = (p1[1] - p2[1]) / 64.0
        b = (p3[0] - p2[0]) / 64.0
        f = (p3[1] - p2[1]) / 64.0
        c = (p4[0] + p2[0] - p1[0] - p3[0]) / 4096.0
        g = (p4[1] + p2[1] - p1[1] - p3[1]) / 4096.0

        # 3. Calculate X and Y coordinates
        X = (a * self.A_grid + b * self.B_grid + c * self.AB_grid + d).astype(np.int32)
        Y = (e * self.A_grid + f * self.B_grid + g * self.AB_grid + h).astype(np.int32)

        # 4. Drawing via Individual Pixels
        for i in range(64):
            for j in range(64):
                self.screen.set_at((X[i, j], Y[i, j]), texture_array[i, j])

    # Shamelessly copied from https://github.com/davidpendergast/pygame-utils/blob/main/warp.py
    def warp(self, surf: pygame.Surface, warp_pts, smooth=True, out: pygame.Surface = None):
        """Stretches a pygame surface to fill a quad using cv2's perspective warp.

            Args:
                surf: The surface to transform.
                warp_pts: A list of four xy coordinates representing the polygon to fill.
                    Points should be specified in clockwise order starting from the top left.
                smooth: Whether to use linear interpolation for the image transformation.
                    If false, nearest neighbor will be used.
                out: An optional surface to use for the final output. If None or not
                    the correct size, a new surface will be made instead.

            Returns:
                [0]: A Surface containing the warped image.
                [1]: A Rect describing where to blit the output surface to make its coordinates
                    match the input coordinates.
        """
        if len(warp_pts) != 4:
            raise ValueError("warp_pts must contain four points")

        w, h = surf.get_size()
        is_alpha = surf.get_flags() & pygame.SRCALPHA

        # XXX throughout this method we need to swap x and y coordinates
        # when we pass stuff between pygame and cv2. I'm not sure why .-.
        src_corners = np.float32([(0, 0), (0, w), (h, w), (h, 0)])
        quad = [tuple(reversed(p)) for p in warp_pts]

        # find the bounding box of warp points
        # (this gives the size and position of the final output surface).
        min_x, max_x = float('inf'), -float('inf')
        min_y, max_y = float('inf'), -float('inf')
        for p in quad:
            min_x, max_x = min(min_x, p[0]), max(max_x, p[0])
            min_y, max_y = min(min_y, p[1]), max(max_y, p[1])
        warp_bounding_box = pygame.Rect(int(min_x), int(min_y),
                                        int(max_x - min_x),
                                        int(max_y - min_y))

        # TODO: Fix this so that single lines are displayed.
        if int(min_x) == int(max_x) or int(min_y) == int(max_y):
            return None, None

        shifted_quad = [(p[0] - min_x, p[1] - min_y) for p in quad]
        dst_corners = np.float32(shifted_quad)

        mat = cv2.getPerspectiveTransform(src_corners, dst_corners)

        orig_rgb = pygame.surfarray.pixels3d(surf)

        flags = cv2.INTER_LINEAR if smooth else cv2.INTER_NEAREST
        out_rgb = cv2.warpPerspective(orig_rgb, mat, warp_bounding_box.size, flags=flags)

        if out is None or out.get_size() != out_rgb.shape[0:2]:
            out = pygame.Surface(out_rgb.shape[0:2], pygame.SRCALPHA if is_alpha else 0)

        pygame.surfarray.blit_array(out, out_rgb)

        if is_alpha:
            orig_alpha = pygame.surfarray.pixels_alpha(surf)
            out_alpha = cv2.warpPerspective(orig_alpha, mat, warp_bounding_box.size, flags=flags)
            alpha_px = pygame.surfarray.pixels_alpha(out)
            alpha_px[:] = out_alpha
        else:
            out.set_colorkey(surf.get_colorkey())

        # XXX swap x and y once again...
        return out, pygame.Rect(warp_bounding_box.y, warp_bounding_box.x,
                                warp_bounding_box.h, warp_bounding_box.w)

    def draw_textured_side(self, surf: pygame.Surface, p1, p2, p3, p4):
        if p1[0] < 0 and p2[0] < 0 and p3[0] < 0 and p4[0] < 0:
            return
        if p1[1] < 0 and p2[1] < 0 and p3[1] < 0 and p4[1] < 0:
            return
        if p1[0] > self.screen_width and p2[0] > self.screen_width and p3[0] > self.screen_width and p4[0] > self.screen_width:
            return
        if p1[1] > self.screen_height and p2[1] > self.screen_height and p3[1] > self.screen_height and p4[1] > self.screen_height:
            return
        out, warp_bounding_box = self.warp(surf, [(p1[0], p1[1]), (p2[0], p2[1]), (p3[0], p3[1]), (p4[0], p4[1])])
        if out is not None:
            self.screen.blit(out, (warp_bounding_box.x, warp_bounding_box.y))

    def get_area_name(self, x, y):
        best_area = None
        best_size = 256 * 256 + 1
        for area in self.cmp.nav_data:
            if area['x'] <= x < area['x'] + area['w'] and area['y'] <= y < area['y'] + area['h']:
                size = area['w'] * area['h']
                if size < best_size:
                    best_size = size
                    best_area = area
        return best_area['name'] if best_area else ""

    def get_slope_heights(self, z, slope_type):
        """ Returns slope heights for the 4 corners of a lid: top-left, top-right, bottom-right, bottom-left. """
        deltas = slope_to_delta[slope_type]
        # The weird order (deltas[2] before deltas[3]) is because slope_to_delta
        # stores the corners of the square in the order TL, TR, BL, BR whereas
        # the rest of the code uses TL, TR, BR, BL.
        return z+deltas[0], z+deltas[1], z+deltas[3], z+deltas[2]

    def vehicle_type_const(self, vtype):
        if vtype == 0:
            return "bus"
        #if vtype == 1:
        #    return "front of juggernaut"
        #if vtype == 2:
        #    return "back of juggernaut"
        if vtype == 3:
            return "bike"
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
        return "car"


    def run(self):
        ped_legends = [
            "Walking", "Running", "Exiting vehicle", "Entering vehicle", "???", "Tumble", "Down", "???", "???", "???", "Punching (still)", "???",
            "???", "???", "???", "Mounting motorcycle", "On motorcycle", "Dismouting motorcycle", "Gun (still)", "Jumping over car", "Driving", "Standing", "Gun (backward)",
            "Gun (forward)", "???", "Flame-thrower (backward)", "Flame-thrower (forward)", "Flame-thrower (still)", "Machine-gun (backward)", "Machine-gun (forward)", "Machine-gun (still)",
            "Bazooka (backward)", "Bazooka (forward)", "Bazooka (still)", "Electrocuted", "Swimming?", "Punching (forward)",
            "Policeman walking", "Policeman running", "Policeman exiting car", "Policeman entering car", "Policeman ???", "Policeman tumbling", "Policman down", "Policeman rising",
            "Pedestrian drowning", "Policeman ???", "Policeman punching", "Pedestrian ???", "Pedestrian walking 1", "Pedestrian walking 2",
            "Policeman ???", "Policeman mounting motorcycle", "Policeman on motorcycle", "Policeman dismounting motorcycle", "Policeman shooting", "Policeman jumping over car",
            "Policeman driving", "Paramedic healing", "Policeman electrocuted", "Policman firing machine gun"
        ]
        ped_grouping = [
            8, 8, 8, 10, 4, 4, 2, 1, 4, 1, 6, 2,
            10, 2, 10, 4, 1, 4, 2, 6, 1, 1, 8,
            8, 3, 8, 8, 2, 8, 8, 2,
            8, 8, 2, 5, 4, 8,
            8, 8, 9, 9, 4, 4, 2, 3,
            2, 1, 6, 2, 7, 5,
            10, 4, 1, 4, 2, 6,
            1, 1, 5, 2
        ]
        ped_boundaries = [sum(ped_grouping[:x]) for x in range(len(ped_grouping))]
        player_sprite = 0
        player_height = 4
        player_rotation = 0
        vrotate = 5 # rotation speed
        player_remap = 0
        anim_tick_start = pygame.time.get_ticks()
        car_speed = 0
        help_text = ["Commands:",
                     " F1: Show this help message",
                     " F2: Switch between display mode and play mode",
                     " i: Show information window",
                     " q/Escape: Quit",
                     " f: toggle fullscreen",
                     "",
                     "In display mode:",
                     " up/down/left/right: move the view in the corresponding direction",
                     " x/c: rotate the player left(counter clockwise)/right(clockwise)",
                     " s: switch between not showing player, showing pedestrian player, showing car player",
                     " p: switch to previous player sprite",
                     " n: switch to next player sprite",
                     " h/H: move player up/down",
                     " m/M: switch to next/previous player remap",
                     " u/d: move camera up/down (a.k.a. dezoom/zoom)",
                     " r: toggle apply remaps",
                     "",
                     "In play mode:",
                     " up: move forward/accelerate",
                     " down: move backward/decelarate",
                     " left/right: turn left(counter clockwise)/right(clockwise)",
                     " c: switch between car/pedestrian",
                     " n/p: switch to next/previous car",
                     ]

        running = True
        show_info = True
        show_help = False
        # 0 = no, 1 = pedestrian, 2 = car
        show_player = 0
        start = None
        frames = 0
        fps = 0.0
        while running:
            ticks = pygame.time.get_ticks()
            if start is None:
                start = ticks
                frames = 0
            end = ticks
            if end-start > 2000:
                fps = 1000*frames/(end-start)
                start = None
            frames += 1
            for event in pygame.event.get():
                if event.type == pygame.QUIT: running = False
                if event.type == pygame.MOUSEBUTTONDOWN:
                    if event.button == 1:
                        # We consider the position at the lowest level.
                        # TODO: Ideally, we would adapt to the level of the tile pressed.
                        cx, cy = self.screen_to_world(event.pos[0], event.pos[1], 5)
                        self.clicked_x, self.clicked_y = int(cx*64), int(cy*64)
                if event.type == pygame.KEYDOWN:
                    if event.key == pygame.K_F1:
                        show_help = not show_help
                    if event.key == pygame.K_F2:
                        self.play_mode = not self.play_mode
                        if self.play_mode:
                            show_player = 1
                    if event.key == pygame.K_i: show_info = not show_info
                    if event.key == pygame.K_ESCAPE or event.key == pygame.K_q: running = False
                    if event.key == pygame.K_f:
                        self.fullscreen = not self.fullscreen
                        self.init_display()
                    if self.play_mode:
                        if event.key == pygame.K_c:
                            show_player = 2 if show_player == 1 else 1
                        if event.key == pygame.K_n:
                            # TODO: Switch to next car model
                            pass
                        if event.key == pygame.K_p:
                            # TODO: Switch to previous car model
                            pass
                    else:
                        if event.key == pygame.K_s:
                            show_player = (show_player + 1) % 3
                            player_sprite = 0
                            player_remap = 0
                            anim_tick_start = pygame.time.get_ticks()
                        if event.key == pygame.K_p:
                            if player_sprite > 0:
                                player_sprite -= 1
                            anim_tick_start = ticks
                        if event.key == pygame.K_n:
                            if (show_player == 1 and player_sprite + 1 < len(ped_grouping)) or (show_player == 2 and player_sprite + 1 < len(self.g24.car_info)):
                                player_sprite += 1
                            anim_tick_start = ticks
                        if event.key == pygame.K_h:
                            if event.mod & pygame.KMOD_SHIFT:
                                if player_height < 5:
                                    player_height += 1
                            else:
                                if player_height > 0:
                                    player_height -= 1
                        if event.key == pygame.K_m:
                            if event.mod & pygame.KMOD_SHIFT:
                                if player_remap > 0:
                                    player_remap -= 1
                            else:
                                if (show_player == 1 and player_remap < 64) or (show_player == 2 and player_remap < 12):
                                    player_remap += 1
                        if event.key == pygame.K_u:
                            # 0.05 is roughly where we start to see the whole map on the screen (at 1024x768)
                            # This is already very slow, no need to let the user go further.
                            if self.base_scale > 0.05:
                                self.base_scale /= 2
                        if event.key == pygame.K_d:
                            # Above 8, things start getting slow because pygame.transform.scale becomes slow.
                            if self.base_scale < 8:
                                self.base_scale *= 2
                        if event.key == pygame.K_r:
                            self.apply_remaps = not self.apply_remaps
            keys = pygame.key.get_pressed()
            if self.play_mode:
                walk_speed = 0.01
                run_speed = 0.03
                if show_player == 1:
                    if keys[pygame.K_LEFT]:
                        player_rotation += vrotate
                    if keys[pygame.K_RIGHT]:
                        player_rotation -= vrotate
                    if keys[pygame.K_UP]:
                        player_sprite = 1  # running
                        angle = 2*math.pi*player_rotation/360
                        dx, dy = run_speed * math.sin(angle), run_speed * math.cos(angle)
                        self.view_x += dx
                        self.view_y += dy
                    if keys[pygame.K_DOWN]:
                        player_sprite = 0  # walking
                        angle = 2*math.pi*player_rotation/360
                        dx, dy = walk_speed * math.sin(angle), walk_speed * math.cos(angle)
                        self.view_x -= dx
                        self.view_y -= dy
                elif show_player == 2:
                    # TODO: Handle commands for driving car
                    # TODO: Support handbrake
                    # Natural braking of the car.
                    if car_speed > 0:
                        car_speed = max(min(car_speed-0.001, car_speed*0.95), 0)
                    elif car_speed < 0:
                        car_speed = min(max(car_speed+0.001, car_speed*0.95), 0)
                    if keys[pygame.K_LEFT]:
                        player_rotation += vrotate * car_speed * 3
                    if keys[pygame.K_RIGHT]:
                        player_rotation -= vrotate * car_speed * 3
                    if keys[pygame.K_UP]:
                        # TODO: Cap the car speed, use car's properties for acceleration, max speed, etc...
                        if car_speed < 0.5:
                            car_speed += 0.03
                    if keys[pygame.K_DOWN]:
                        if car_speed > -0.2:
                            car_speed -= 0.02
                    angle = 2*math.pi*player_rotation/360
                    dx, dy = car_speed * math.sin(angle), car_speed * math.cos(angle)
                    self.view_x += dx
                    self.view_y += dy
                # TODO: Zoom/dezoom depending on the speed
                # TODO: React based on the tile: slow down on fields, crash on buildings, go up/down on slopes, ...
            else:
                move_speed = 1
                if keys[pygame.K_LEFT]: self.view_x -= move_speed
                if keys[pygame.K_RIGHT]: self.view_x += move_speed
                if keys[pygame.K_UP]: self.view_y -= move_speed
                if keys[pygame.K_DOWN]: self.view_y += move_speed
                if keys[pygame.K_x]: player_rotation += vrotate
                if keys[pygame.K_c]: player_rotation -= vrotate
            if player_rotation > 360:
                player_rotation -= 360
            if player_rotation < -360:
                player_rotation += 360
            # Alternative way to handle the zoom, useful for a more progressive one.
            #if keys[pygame.K_u] and self.base_scale > 0.05: self.base_scale /= 1.01
            #if keys[pygame.K_d] and self.base_scale < 8: self.base_scale *= 1.01
            self.screen.fill((0, 0, 0))
            vx_int, vy_int = int(self.view_x), int(self.view_y)
            # Margin to handle parallax bringing blocks from sides
            margin = 1
            range_x, range_y = int(self.display_tiles_h / self.base_scale), int(self.display_tiles_v / self.base_scale)
            for z in reversed(range(self.min_z, self.max_z)):
                min_x, min_y = self.screen_to_world(0, 0, z)
                max_x, max_y = self.screen_to_world(self.screen_width, self.screen_height, z)
                min_x, min_y = int(min_x-margin), int(min_y-margin)
                max_x, max_y = int(max_x+margin+1), int(max_y+margin+1)
                if self.show_tiles:
                    for step in ['sides', 'lid']:
                        for y in range(min_y, max_y):
                            if y < 0 or y >= 256: continue
                            for x in range(min_x, max_x):
                                if x < 0 or x >= 256: continue
                                height, blocks = self.cmp.get_column(x, y)
                                delta = 6-len(blocks)
                                if (z - delta) >= 0 and (z-delta) < len(blocks):
                                    block = blocks[z-delta]
                                    z1, z2, z3, z4 = self.get_slope_heights(z, block['slope'])
                                    c1, c2, c3, c4 = self.world_to_screen(x,y,z1), self.world_to_screen(x+1,y,z2), self.world_to_screen(x+1,y+1,z3), self.world_to_screen(x,y+1,z4)
                                    if step == 'sides' and self.show_sides and z < 5:
                                        b1, b2, b3, b4 = self.world_to_screen(x,y,z+1), self.world_to_screen(x+1,y,z+1), self.world_to_screen(x+1,y+1,z+1), self.world_to_screen(x,y+1,z+1)
                                        if block['top'] > 0:
                                            self.draw_textured_side(pygame.transform.flip(self.get_tile_surface('side', block['top'], ticks), block['flip_top_bottom'], 0), c1, c2, b2, b1)
                                        if block['bottom'] > 0 and not block['flat']:
                                            self.draw_textured_side(pygame.transform.flip(self.get_tile_surface('side', block['bottom'], ticks), block['flip_top_bottom'], 0), c4, c3, b3, b4)
                                        if block['left'] > 0:
                                            self.draw_textured_side(pygame.transform.flip(self.get_tile_surface('side', block['left'], ticks), block['flip_left_right'], 0), c1, c4, b4, b1)
                                        if block['right'] > 0 and not block['flat']:
                                            self.draw_textured_side(pygame.transform.flip(self.get_tile_surface('side', block['right'], ticks), block['flip_left_right'], 0), c2, c3, b3, b2)
                                    if step == 'lid' and self.show_lids:
                                        if block['lid'] > 0:
                                            lid_remap = block['lid_remap']
                                            if not self.apply_remaps:
                                                lid_remap = 0
                                            surf = self.get_tile_surface('lid', block['lid'], ticks, lid_remap)
                                            if surf:
                                                if block['lid_rotation'] != 0: surf = pygame.transform.rotate(surf, -90 * block['lid_rotation'])
                                                w, h = int(c2[0]-c1[0])+1, int(c4[1]-c1[1])+1
                                                if block['slope'] != 0:
                                                    # I'm not sure why this is needed, but without the 'spill', there's a black border around some of the blocks with slopes.
                                                    # This doesn't fully fix the issue but this is the best result I managed so far.
                                                    spill = 2*self.base_scale
                                                    c1 = int(c1[0]), int(c1[1])
                                                    c2 = int(c2[0]+spill), int(c2[1])
                                                    c3 = int(c3[0]+spill), int(c3[1]+spill)
                                                    c4 = int(c4[0]), int(c4[1]+spill)
                                                    self.draw_textured_side(surf, c1, c2, c3, c4)
                                                else:
                                                    if w > 0 and h > 0:
                                                        self.screen.blit(pygame.transform.scale(surf, (w, h)), (int(c1[0]), int(c1[1])))
                                                    else:
                                                        print(f"WARNING: Unexpected width & height for lid: {w},{h}")
                if self.show_objects:
                    for obj in self.cmp.objects:
                        ox, oy, oz = obj['x']/64.0, obj['y']/64.0, (obj['z']+1)/64.0
                        if int(oz) == z:
                            if min_x < ox < max_x and min_y < oy < max_y:
                                sx, sy, scale = self.world_to_screen(ox, oy, oz)
                                spr_num = -1
                                if obj['remap'] >= 128:
                                    info = next(car for car in self.g24.car_info if car['model'] == obj['type'])
                                    base_name = 'car'
                                    if info['vtype'] == 0: base_name = 'bus'
                                    elif info['vtype'] == 3: base_name = 'bike'
                                    elif info['vtype'] == 8: base_name = 'train'
                                    elif info['vtype'] == 9: base_name = 'tram'
                                    elif info['vtype'] == 13: base_name = 'boat'
                                    elif info['vtype'] == 14: base_name = 'tank'
                                    spr_num = self.g24.sprite_bases.get(base_name, 0) + info['spr_num']
                                else:
                                    o_idx = obj['type']
                                    if o_idx < len(self.g24.object_info):
                                        info = self.g24.object_info[o_idx]
                                        if info['status'] == 3:  # invisible
                                            continue
                                        spr_num = self.g24.sprite_bases.get('object', 0) + info['spr_num']
                                        if info['status'] == 5 or info['status'] == 9:
                                            frames = 8
                                            if info['status'] == 5:
                                                frames = info['width']
                                            speed = info['height']
                                            frame_idx = (ticks // max(1, (speed * 1000 // 60))) % frames
                                            spr_num += frame_idx
                                    else:
                                        print(f"ERROR: Object not found: {o_idx}")
                                if spr_num >= 0:
                                    spr_surf = self.get_sprite_surface(spr_num, remap=-1)
                                    if spr_surf:
                                        scaled = pygame.transform.scale(spr_surf, (max(1, int(spr_surf.get_width()*scale)), max(1, int(spr_surf.get_height()*scale))))
                                        rotated = pygame.transform.rotate(scaled, obj['rotation'] * 90 / 256)
                                        self.screen.blit(rotated, (int(sx - rotated.get_width()/2), int(sy - rotated.get_height()/2)))
                if show_player > 0 and z == player_height:
                    sx, sy, scale = self.world_to_screen(self.view_x + 10, self.view_y + 8, player_height)
                    if show_player == 1:
                        anim_speed = 2 # works well (at least for walking/running)
                        anim_idx = ((ticks - anim_tick_start) // (1000 * anim_speed // 20)) % ped_grouping[player_sprite]
                        spr_num = self.g24.sprite_bases['ped'] + ped_boundaries[player_sprite] + anim_idx
                        if 'newcarclut_size' in self.g24.header:
                            base_remap = self.g24.header['newcarclut_size'] // 1024 - 64  # There are 64 remaps for pedestrian and they are at the end of the newcarclut section
                        else:
                            base_remap = 125
                        remap = -1
                        if player_remap > 0:
                            remap = base_remap+player_remap-1
                        spr_surf = self.get_sprite_surface(spr_num, remap)
                    elif show_player == 2:
                        car_info = self.g24.car_info[player_sprite]
                        spr_num = car_info['spr_num'] + self.g24.sprite_bases[self.vehicle_type_const(car_info['vtype'])]
                        base_remap = player_sprite * 12  # There are 12 remaps per car
                        remap = -1
                        if player_remap > 0:
                            remap = base_remap+player_remap-1
                        spr_surf = self.get_sprite_surface(spr_num, remap)
                    if spr_surf:
                        scaled = pygame.transform.scale(spr_surf, (max(1, int(spr_surf.get_width()*scale)), max(1, int(spr_surf.get_height()*scale))))
                        rotated = pygame.transform.rotate(scaled, player_rotation)
                        self.screen.blit(rotated, (int(sx - rotated.get_width()/2), int(sy - rotated.get_height()/2)))



            # Code to find where blocks with a particular property are
            if False:
                for y in range(0, 256):
                    for x in range(0, 256):
                        _, blocks = self.cmp.get_column(x, y)
                        for block in blocks:
                            if block['lid_remap'] != 0:
                                print(f"Remapped lid at {x},{y}")
                break
            # Code to find where objects with a particular property are
            if False:
                for obj in self.cmp.objects:
                    if obj['remap'] > 0 and obj['remap'] < 128:
                        info = self.g24.object_info[obj['type']]
                        x = obj['x'] // 64
                        y = obj['y'] // 64
                        print(f"Remapped object at {x},{y}: {obj} - {info}")
                    if obj['remap'] > 128:
                        info = next(car for car in self.g24.car_info if car['model'] == obj['type'])
                        x = obj['x'] // 64
                        y = obj['y'] // 64
                        print(f"Remapped car at {x},{y}: {obj} - {info}")
                break


            # Display area name
            # TODO: Understand the logic the game uses to split areas in North/South East/West and Central: is it just splitting the area in 9? Is it more subtle?
            area_name = self.get_area_name(int(self.view_x + 10), int(self.view_y + 8))
            if area_name:
                img = self.font.render(area_name, True, (255, 255, 0))
                bg = pygame.Surface((img.get_width() + 40, img.get_height() + 40))
                bg.fill((0, 0, 0))
                bg.set_alpha(100)
                self.screen.blit(bg, (self.screen_width // 2 - img.get_width() // 2 - 20, self.screen_height - 50 - 20))
                self.screen.blit(img, (self.screen_width // 2 - img.get_width() // 2, self.screen_height - 50))

            if show_help:
                lines = [self.font.render(line, True, (255, 255, 255)) for line in help_text]
                width = max([l.get_width() for l in lines])
                height = 40 + sum([l.get_height() for l in lines])
                bg = pygame.Surface((width, height))
                bg.fill((0, 0, 0))
                bg.set_alpha(180)
                self.screen.blit(bg, (10, 10))
                dy = 0
                for l in lines:
                    self.screen.blit(l, (30, 30+dy))
                    dy += l.get_height()
            elif show_info:
                mode = "Play" if self.play_mode else "Display"
                text1 = f"Mode: {mode}  X: {self.view_x:.0f} Y: {self.view_y:.0f}  FPS: {fps:.2f}  zoom: {100*self.base_scale}"
                text2 = f"Remaps: {self.apply_remaps} - Clicked pos (tile): {self.clicked_x}, {self.clicked_y} ({self.clicked_x//64}, {self.clicked_y//64})"
                text3 = ""
                if show_player == 1:
                    text3 = f"Player: remap: {player_remap} - height: {player_height} - sprite: {ped_legends[player_sprite]} - rotation: {player_rotation}"
                if show_player == 2:
                    text3 = f"Car: remap: {player_remap} - height: {player_height} - rotation: {player_rotation}"
                img1 = self.font.render(text1, True, (255, 255, 255))
                img2 = self.font.render(text2, True, (255, 255, 255))
                img3 = self.font.render(text3, True, (255, 255, 255))
                width = max(img1.get_width(), img2.get_width()) + 40
                height = img1.get_height() + img2.get_height() + 50
                if show_player:
                    width = max(width, img3.get_width() + 40)
                    height += img3.get_height() + 10
                bg = pygame.Surface((width, height))
                bg.fill((0, 0, 0))
                bg.set_alpha(180)
                self.screen.blit(bg, (10, 10))
                self.screen.blit(img1, (30, 30))
                self.screen.blit(img2, (30, 30+img1.get_height() + 10))
                if show_player:
                    self.screen.blit(img3, (30, 60+img1.get_height() + 20))
            pygame.display.flip()
            self.clock.tick(60)
        pygame.quit()

def resolution(arg):
    parts = arg.split('x')
    if len(parts) != 2:
        raise argparse.ArgumentTypeError(f"Invalid resolution '{arg}', want <width>x<height>")
    try:
        return int(parts[0]), int(parts[1])
    except:
        raise argparse.ArgumentTypeError(f"Invalid resolution '{arg}', want <width>x<height>")

profile = False

if __name__ == '__main__':
    if profile:
        import cProfile, pstats, io
        pr = cProfile.Profile()
        pr.enable()

    parser = argparse.ArgumentParser(description='Display a GTA map')
    parser.add_argument('cmp_file', help='Input CMP file')
    parser.add_argument('g24_file', help='Input G24 file')
    parser.add_argument('--no_objects', '-o', action='store_true', help='Do not show objects')
    parser.add_argument('--no_cars', '-c', action='store_true', help='Do not show cars')
    parser.add_argument('--no_tiles', '-t', action='store_true', help='Do not show tiles')
    parser.add_argument('--no_sides', '-s', action='store_true', help='Do not show sides')
    parser.add_argument('--no_lids', '-l', action='store_true', help='Do not show lids')
    parser.add_argument('--min_z', '-z', type=int, default=0, help='Minimum z to show')
    parser.add_argument('--max_z', '-Z', type=int, default=6, help='Maximum z to show')
    parser.add_argument('--resolution', '-r', type=resolution, default='1024x768', help='Screen resolution')
    parser.add_argument('--fullscreen', '-f', action='store_true', help='Fullscreen mode')

    args = parser.parse_args()

    renderer = MapRenderer(args.cmp_file, args.g24_file, show_objects=not args.no_objects, show_tiles=not args.no_tiles, show_sides=not args.no_sides, show_lids=not args.no_lids, min_z=args.min_z, max_z=args.max_z, width=args.resolution[0], height=args.resolution[1], fullscreen=args.fullscreen)
    renderer.run()

    if profile:
        pr.disable()
        s = io.StringIO()
        sortby = 'cumulative'
        ps = pstats.Stats(pr, stream=s).sort_stats(sortby)
        ps.print_stats()
        print(s.getvalue())
