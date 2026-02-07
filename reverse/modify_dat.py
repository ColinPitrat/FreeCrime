import struct
import argparse
import sys

class DATFile:
    def __init__(self, filepath):
        self.filepath = filepath
        with open(filepath, 'rb') as f:
            self.data = bytearray(f.read())
        self.parse()

    def parse(self):
        # Header (18 bytes)
        self.sound_volume = self.data[0]
        self.music_volume = self.data[1]
        self.text_speed = self.data[2]
        self.radio_mode = self.data[3]
        self.resolution = self.data[4]
        self.default_resolution = self.data[5]
        self.reserved1 = self.data[6]
        self.transparency_effects = self.data[7]
        self.deathmatch_mode = self.data[8]
        self.deathmatch_score = struct.unpack_from('<I', self.data, 9)[0]
        self.deathmatch_kills = struct.unpack_from('<I', self.data, 13)[0]
        self.language = self.data[17]

        # Determine version/counts based on file size
        self.num_highscores = 18
        self.highscore_name_size = 15
        self.highscore_size = self.highscore_name_size + 4
        self.num_players = 8
        self.player_name_size = 15
        self.player_size = 64 + self.player_name_size

        self.highscores = []
        offset = 18
        for _ in range(self.num_highscores):
            score = struct.unpack_from('<I', self.data, offset)[0]
            name = self.data[offset+4:offset+4+self.highscore_name_size].split(b'\x00')[0].decode('ascii', errors='ignore')
            self.highscores.append({'score': score, 'name': name})
            offset += self.highscore_size

        self.players = []
        for _ in range(self.num_players):
            p_data = self.data[offset:offset+self.player_size]
            name = p_data[:self.player_name_size].split(b'\x00')[0].decode('ascii', errors='ignore')
            p_highscores = list(struct.unpack_from('<6i', p_data, self.player_name_size))
            off = self.player_name_size + 24
            reserved2 = struct.unpack_from('<i', p_data, off)[0]
            area = struct.unpack_from('<i', p_data, off+4)[0]
            reserved3 = struct.unpack_from('<i', p_data, off+8)[0]
            game = struct.unpack_from('<i', p_data, off+12)[0]
            video = list(struct.unpack_from('<6I', p_data, off+16))

            self.players.append({
                'name': name,
                'highscores': p_highscores,
                'reserved2': reserved2,
                'multiplayer_selected_area': area,
                'reserved3': reserved3,
                'multiplayer_selected_game': game,
                'video_unlocked': video
            })
            offset += self.player_size

        # Trailer
        self.selected_player_id = self.data[offset]
        self.multiplayer_host_name = self.data[offset+1:offset+17].split(b'\x00')[0].decode('ascii', errors='ignore')

    def pack(self):
        res = bytearray()
        res.append(self.sound_volume)
        res.append(self.music_volume)
        res.append(self.text_speed)
        res.append(self.radio_mode)
        res.append(self.resolution)
        res.append(self.default_resolution)
        res.append(self.reserved1)
        res.append(self.transparency_effects)
        res.append(self.deathmatch_mode)
        res.extend(struct.pack('<I', self.deathmatch_score))
        res.extend(struct.pack('<I', self.deathmatch_kills))
        res.append(self.language)

        for hs in self.highscores:
            res.extend(struct.pack('<I', hs['score']))
            name_bytes = hs['name'].encode('ascii')[:self.highscore_name_size]
            res.extend(name_bytes + b'\x00' * (self.highscore_name_size - len(name_bytes)))

        res.extend(self.gap)

        for p in self.players:
            p_data = bytearray()
            name_bytes = p['name'].encode('ascii')[:self.player_name_size]
            p_data.extend(name_bytes + b'\x00' * (self.player_name_size - len(name_bytes)))
            p_data.extend(struct.pack('<6i', *p['highscores']))
            p_data.extend(struct.pack('<i', p['reserved2']))
            p_data.extend(struct.pack('<i', p['multiplayer_selected_area']))
            p_data.extend(struct.pack('<i', p['reserved3']))
            p_data.extend(struct.pack('<i', p['multiplayer_selected_game']))
            p_data.extend(struct.pack('<6I', *p['video_unlocked']))
            res.extend(p_data)

        res.append(self.selected_player_id)
        name_bytes = self.multiplayer_host_name.encode('ascii')[:16]
        res.extend(name_bytes + b'\x00' * (16 - len(name_bytes)))

        return res

    def save(self, filepath):
        with open(filepath, 'wb') as f:
            f.write(self.pack())

    def display(self):
        print(f"Sound Volume: {self.sound_volume}")
        print(f"Music Volume: {self.music_volume}")
        print(f"Text Speed: {self.text_speed}")
        print(f"Radio Mode: {self.radio_mode}")
        print(f"Resolution: {self.resolution}")
        print(f"Default Resolution: {self.default_resolution}")
        print(f"Reserved 1: {self.reserved1}")
        print(f"Transparency Effects: {self.transparency_effects}")
        print(f"Deathmatch Mode: {self.deathmatch_mode}")
        print(f"Deathmatch Score: {self.deathmatch_score}")
        print(f"Deathmatch Kills: {self.deathmatch_kills}")
        print(f"Language: {self.language}")
        print(f"Selected Player ID: {self.selected_player_id}")
        print(f"Multiplayer Host Name: {self.multiplayer_host_name}")

        print("\nHigh Scores:")
        for n, city in enumerate(["Liberty City", "San Andreas", "Vice City"]):
            for level in range(2):
                print(f"  - {city:12} {level+1}:")
                for place in [1, 2, 3]:
                    hs = self.highscores[6*n+3*level+3-place]
                    print(f"     - {place}: {hs['name']:16} - {hs['score']}")
        #for i, hs in enumerate(self.highscores):
        #    print(f"  {i+1:2d}: {hs['score']:8d} - {hs['name']}")

        print("\nPlayers:")
        for i, p in enumerate(self.players):
            print(f"  Player {i+1}: {p['name']}")
            print(f"    Highscores: {p['highscores']}")
            print(f"    Area: {p['multiplayer_selected_area']}, Game: {p['multiplayer_selected_game']}")
            print(f"    Reserved 2: {p['reserved2']}, Reserved 3: {p['reserved3']}")
            print(f"    Videos: {p['video_unlocked']}")

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
    parser = argparse.ArgumentParser(description='Modify DAT files')
    parser.add_argument('input_file', help='Input DAT file')
    parser.add_argument('--output', '-o', help='Output DAT file (default: overwrite input)')
    parser.add_argument('--set', '-s', action='append', help='Set field value, e.g. sound_volume=5')
    parser.add_argument('--print', '-p', action='append', help='Print field value, e.g. players[0].name')
    parser.add_argument('--display', '-d', action='store_true', help='Display all content')

    args = parser.parse_args()

    dat_file = DATFile(args.input_file)

    if args.display:
        dat_file.display()

    if args.print:
        for p in args.print:
            parent, key = resolve_path(dat_file, p)
            try:
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
            parent, key = resolve_path(dat_file, path)

            try:
                current_val = get_item(parent, key)
                target_type = type(current_val)
                if target_type == int:
                    new_val = int(value)
                elif target_type == float:
                    new_val = float(value)
                elif target_type == str:
                    new_val = value
                elif target_type == list:
                    new_val = [int(v.strip()) for v in value.split(',')]
                else:
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
        dat_file.save(output_path)
        print(f"Saved to {output_path}")

if __name__ == '__main__':
    main()
