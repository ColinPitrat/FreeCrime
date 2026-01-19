#!/usr/bin/env python3
import sys
import os
import argparse
import numpy as np
import pygame

def load_palette(filename):
    """
    Load palette from a matching .ACT file or fallback to F_PAL.RAW.
    """
    act_path = os.path.splitext(filename)[0] + '.ACT'
    if os.path.exists(act_path):
        with open(act_path, 'rb') as f:
            data = f.read(768)
            return np.frombuffer(data, dtype=np.uint8).reshape(-1, 3)

    img_dir = os.path.dirname(filename)
    f_pal_path = os.path.join(img_dir, 'F_PAL.RAW')
    if os.path.exists(f_pal_path):
        with open(f_pal_path, 'rb') as f:
            data = f.read(768)
            return np.frombuffer(data, dtype=np.uint8).reshape(-1, 3)

    if os.path.exists('F_PAL.RAW'):
        with open('F_PAL.RAW', 'rb') as f:
            data = f.read(768)
            return np.frombuffer(data, dtype=np.uint8).reshape(-1, 3)

    return np.arange(256, dtype=np.uint8).repeat(3).reshape(-1, 3)

def main():
    parser = argparse.ArgumentParser(description='Display RAW/RAT images with varying width using Pygame.')
    parser.add_argument('filenames', nargs='+', help='The RAW or RAT file to display')
    args = parser.parse_args()

    for filename in args.filenames:
        error = False
        if not os.path.exists(filename):
            print(f"File not found: {filename}")
            error = True
        if error:
            sys.exit(1)

    pygame.init()
    screen = pygame.display.set_mode((1920, 1440), pygame.RESIZABLE)

    def load_file():
        global is_rat, palette, raw_data
        filename = args.filenames[fileindex]
        pygame.display.set_caption(f"Image Viewer - {filename}")
        ext = os.path.splitext(filename)[1].upper()
        with open(filename, 'rb') as f:
            raw_data = f.read()

        is_rat = (ext == '.RAT')
        palette = load_palette(filename) if is_rat else None
        return raw_data, is_rat, palette

    fileindex = 1
    raw_data, is_rat, palette = load_file()

    width = 640
    last_width = -1
    running = True
    clock = pygame.time.Clock()

    zoom = 4
    while running:
        # Handle Events
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_LEFT:
                    width = width - 1
                    while len(raw_data) % width != 0:
                        width = width - 1
                        if width <= 0:
                            width = len(raw_data)
                    print(f"Width: {width}")
                elif event.key == pygame.K_RIGHT:
                    width = width + 1
                    while len(raw_data) % width != 0:
                        width = width + 1
                        if width >= len(raw_data):
                            width = 1
                    print(f"Width: {width}")
                elif event.key == pygame.K_e:
                    if screen:
                        export_name = filename + ".BMP"
                        pygame.image.save(screen, export_name)
                        print(f"Saved '{export_name}'")
                elif event.key == pygame.K_z:
                    zoom += 1
                    print(f"Zoom: {zoom}")
                elif event.key == pygame.K_d:
                    if zoom > 1:
                        zoom -= 1
                        print(f"Zoom: {zoom}")
                elif event.key == pygame.K_i:
                    print(f"Width: {width} - Height: {len(raw_data)/width} - Zoom: {zoom}")
                elif event.key == pygame.K_n:
                    if fileindex+1 < len(args.filenames):
                        fileindex += 1
                    raw_data, is_rat, palette = load_file()
                elif event.key == pygame.K_p:
                    if fileindex > 0:
                        fileindex -= 1
                    raw_data, is_rat, palette = load_file()
                elif event.key == pygame.K_q or event.key == pygame.K_ESCAPE:
                    running = False

        # Calculate dimensions
        num_pixels = len(raw_data) // 3 if not is_rat else len(raw_data)
        height = max(1, num_pixels // width)

        visible_count = width * height

        # Prepare Image Surface
        try:
            if not is_rat:
                # RAW 24-bit (RGB)
                img_data = raw_data[:visible_count * 3]
                # Pygame expects (width, height) for surfaces
                # We use frombuffer then reshape to (height, width, 3)
                arr = np.frombuffer(img_data, dtype=np.uint8).reshape((height, width, 3))
                # Swap axes because pygame.surfarray uses (width, height)
                surf = pygame.surfarray.make_surface(arr.swapaxes(0, 1))
            else:
                # RAT 8-bit (Indexed)
                indices = np.frombuffer(raw_data[:visible_count], dtype=np.uint8).reshape((height, width))
                rgb_arr = palette[indices]
                surf = pygame.surfarray.make_surface(rgb_arr.swapaxes(0, 1))

            surf = pygame.transform.scale_by(surf, zoom)

            screen.fill((0,0,0))
            screen.blit(surf, (0, 0))
            pygame.display.flip()
        except Exception as e:
            # Handle potential reshape/surface errors silently
            pass

    pygame.quit()

if __name__ == '__main__':
    main()
