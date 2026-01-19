#!/usr/bin/env python3
import sys
import os
import struct
import argparse
import numpy as np
import pygame

def decode_fon(filename):
    """
    Decodes a .FON file and returns a list of images (as numpy arrays) and the palette.
    """
    with open(filename, 'rb') as f:
        data = f.read()

    if len(data) < 2:
        return [], None

    num_pictures = data[0]
    height = data[1]

    # Palette is at the end (last 768 bytes)
    palette_data = data[-768:]
    palette = np.frombuffer(palette_data, dtype=np.uint8).reshape(256, 3)

    images = []
    offset = 2
    for i in range(num_pictures):
        if offset >= len(data) - 768:
            break
        width = data[offset]
        offset += 1
        size = width * height
        if offset + size > len(data) - 768:
            break
        pixels = np.frombuffer(data[offset:offset+size], dtype=np.uint8).reshape(height, width)
        images.append(pixels)
        offset += size

    return images, palette, height

def save_bmp(filename, pixels, palette):
    """
    Saves a single image as a BMP file using pygame.
    """
    height, width = pixels.shape
    rgb_pixels = palette[pixels]
    # Swap axes for pygame (width, height, 3)
    surf = pygame.surfarray.make_surface(rgb_pixels.swapaxes(0, 1))
    pygame.image.save(surf, filename)

def main():
    parser = argparse.ArgumentParser(description='Decode and display/export .FON files.')
    parser.add_argument('filename', help='The .FON file to process')
    parser.add_argument('--export', help='Directory to export images to')
    args = parser.parse_args()

    if not os.path.exists(args.filename):
        print(f"File not found: {args.filename}")
        sys.exit(1)

    images, palette, height = decode_fon(args.filename)
    if not images:
        print("No images found in file.")
        sys.exit(1)

    # Export if requested
    if args.export:
        if not os.path.exists(args.export):
            os.makedirs(args.export)
        base_name = os.path.splitext(os.path.basename(args.filename))[0]

        # We need pygame initialized for save_bmp
        pygame.display.init()
        # Create a hidden window or just use the functionality if possible
        # Actually pygame.image.save might work without a full window if initialized
        pygame.display.set_mode((1, 1), pygame.NOFRAME)

        for i, img in enumerate(images):
            out_path = os.path.join(args.export, f"{base_name}_{i:03d}.bmp")
            save_bmp(out_path, img, palette)
            print(f"Exported {out_path}")

        pygame.quit()
        return

    # Display mode
    pygame.init()

    current_idx = 0
    running = True
    clock = pygame.time.Clock()

    screen = pygame.display.set_mode((1024, 768), pygame.RESIZABLE)

    zoom = 4
    def update_display(idx):
        nonlocal screen
        img = images[idx]
        h, w = img.shape
        rgb_pixels = palette[img]
        # Pygame uses (width, height)
        surf = pygame.surfarray.make_surface(rgb_pixels.swapaxes(0, 1))
        surf = pygame.transform.scale_by(surf, zoom)

        pygame.display.set_caption(f"FON Viewer - {args.filename} - Image {idx+1}/{len(images)}")
        screen.fill((128,128,128))
        screen.blit(surf, (0, 0))
        pygame.display.flip()

    update_display(current_idx)

    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_n: # Next
                    current_idx = (current_idx + 1) % len(images)
                    update_display(current_idx)
                elif event.key == pygame.K_p: # Previous
                    current_idx = (current_idx - 1) % len(images)
                    update_display(current_idx)
                elif event.key == pygame.K_q or event.key == pygame.K_ESCAPE:
                    running = False
                elif event.key == pygame.K_z:
                    zoom += 1
                elif event.key == pygame.K_d:
                    if zoom > 1:
                        zoom -= 1

        clock.tick(30)

    pygame.quit()

if __name__ == '__main__':
    main()
