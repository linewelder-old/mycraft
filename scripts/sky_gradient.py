import pygame
from math import pow, cos, pi, sin


TEXTURE_SIZE = 1024
WINDOW_SIZE = 512

NIGHT_SKY_COLOR = (0.00, 0.00, 0.08)
DAY_SKY_COLOR   = (0.43, 0.77, 0.98)

NIGHT_FOG_COLOR = (0.00, 0.00, 0.09)
DAY_FOG_COLOR   = (0.53, 0.81, 0.98)

SUNSET_COLOR = (1.00, 0.60, 0.40)


def mix_colors(color1, color2, t):
    return tuple(color1[i] + (color2[i] - color1[i]) * t for i in range(3))


def to_255_rgb(color):
    return tuple(int(color[i] * 255) for i in range(3))


def smoothstep(a, b, t):
    t = (t - a) / (b - a)
    if t < 0.:
        t = 0.
    elif t > 1.:
        t = 1.

    return t * t * (3. - 2. * t)


def generate_sky_slice(texture, x, time):
    dayness = (sin(.5 * pi * cos(time * pi)) + 1.) / 2.
    dayness *= dayness
    sky_color = mix_colors(NIGHT_SKY_COLOR, DAY_SKY_COLOR, dayness)
    fog_color = mix_colors(NIGHT_FOG_COLOR, DAY_FOG_COLOR, dayness)

    y = 10 * (time - 0.4)
    sunset_intensity = max(0., y * y * (3. - 2. * y)) if 0.4 < time < 0.6 else 0.
    fog_color = mix_colors(fog_color, SUNSET_COLOR, sunset_intensity)

    for y in range(TEXTURE_SIZE):
        distance_from_horizon = abs(y - TEXTURE_SIZE / 2.) / TEXTURE_SIZE * 2.
        skyness = smoothstep(0.15, 0.4, distance_from_horizon)
        color = mix_colors(fog_color, sky_color, skyness)

        texture.set_at((x, y), to_255_rgb(color))


if __name__ == "__main__":
    pygame.init()
    screen = pygame.display.set_mode((WINDOW_SIZE, WINDOW_SIZE))
    clock = pygame.time.Clock()
    running = True

    texture = pygame.Surface((TEXTURE_SIZE, TEXTURE_SIZE))
    for x in range(TEXTURE_SIZE):
        generate_sky_slice(texture, x, x / TEXTURE_SIZE)
    pygame.image.save(texture, "sky.png")

    transformed_texture = pygame.transform.scale(texture, (WINDOW_SIZE, WINDOW_SIZE))

    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False

        screen.blit(transformed_texture, (0, 0))

        pygame.display.flip()
        clock.tick(60)

    pygame.quit()