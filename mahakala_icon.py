from PIL import Image

# 创建64x64像素图像
size = 64
img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
pixels = img.load()

# 颜色定义
DARK_BLUE = (15, 15, 50, 255)
GOLD = (255, 215, 0, 255)
RED = (220, 30, 30, 255)
WHITE = (240, 240, 245, 255)
DARK_RED = (160, 20, 20, 255)
ORANGE = (255, 140, 0, 255)
EYE_RED = (255, 50, 50, 255)
PINK = (255, 180, 180, 255)
GOLD2 = (255, 200, 50, 255)

# 火焰背光
for y in range(size):
    for x in range(size):
        cx, cy = 32, 32
        dx, dy = x - cx, y - cy
        dist = max(abs(dx), abs(dy))
        if 25 <= dist <= 30:
            if (x + y) % 3 == 0 or (x - y) % 3 == 0:
                col = ORANGE if dist > 28 else RED
                pixels[x, y] = col
        if 22 <= dist <= 24:
            if (x + y) % 4 == 0:
                pixels[x, y] = (255, 100, 0, 200)

def circle(cx, cy, r, color):
    for y in range(cy-r, cy+r+1):
        for x in range(cx-r, cx+r+1):
            if 0 <= x < size and 0 <= y < size:
                if (x-cx)**2 + (y-cy)**2 <= r**2:
                    if pixels[x, y] == (0, 0, 0, 0) or pixels[x, y][3] < 255:
                        pixels[x, y] = color

# 头部
circle(32, 18, 8, DARK_BLUE)

# 身体
for y in range(26, 50):
    width = int(6 + (y-26) * 0.2)
    for x in range(32-width, 32+width+1):
        if 0 <= x < size:
            if pixels[x, y] == (0, 0, 0, 0):
                pixels[x, y] = DARK_BLUE

# 五骷髅冠
crown_pos = [(24, 11), (28, 9), (32, 8), (36, 9), (40, 11)]
for sx, sy in crown_pos:
    for dy in range(-2, 3):
        for dx in range(-2, 3):
            px, py = sx+dx, sy+dy
            if 0 <= px < size and 0 <= py < size:
                if abs(dx) + abs(dy) <= 2:
                    if dx*dx + dy*dy <= 2:
                        pixels[px, py] = WHITE
                    elif abs(dx) <= 1 and abs(dy) <= 1:
                        pixels[px, py] = GOLD

# 三目 - 左眼
for dy in range(-1, 2):
    for dx in range(-1, 2):
        if 0 <= 27+dx < size and 0 <= 17+dy < size:
            pixels[27+dx, 17+dy] = EYE_RED if dx==0 and dy==0 else WHITE
# 右眼
for dy in range(-1, 2):
    for dx in range(-1, 2):
        if 0 <= 37+dx < size and 0 <= 17+dy < size:
            pixels[37+dx, 17+dy] = EYE_RED if dx==0 and dy==0 else WHITE
# 第三眼
pixels[32, 14] = EYE_RED
pixels[32, 15] = WHITE

# 愤怒眉
for x in range(24, 41):
    if 0 <= x < size:
        y = 13
        pixels[x, y] = GOLD

# 嘴和獠牙
for x in range(29, 36):
    for y in range(22, 24):
        if 0 <= x < size and 0 <= y < size:
            pixels[x, y] = DARK_RED
pixels[28, 21] = WHITE
pixels[28, 22] = WHITE
pixels[36, 21] = WHITE
pixels[36, 22] = WHITE

# 耳环
for y in range(16, 21):
    if 0 <= y < size:
        pixels[23, y] = GOLD
        pixels[41, y] = GOLD

# 项链
for x in range(25, 40):
    if x % 2 == 0:
        if 0 <= x < size and 26 < size:
            pixels[x, 26] = GOLD

# 右手持钺刀
for y in range(27, 34):
    if 0 <= 44 < size and 0 <= y < size:
        pixels[44, y] = DARK_BLUE
for y in range(24, 31):
    for x in range(46, 50):
        if 0 <= x < size and 0 <= y < size:
            if y < 27:
                pixels[x, y] = GOLD
            else:
                pixels[x, y] = GOLD2

# 左手持颅器
for y in range(27, 34):
    if 0 <= 20 < size and 0 <= y < size:
        pixels[20, y] = DARK_BLUE
for y in range(24, 29):
    for x in range(16, 21):
        if 0 <= x < size and 0 <= y < size:
            if (x-18)**2 + (y-26)**2 <= 5:
                pixels[x, y] = WHITE

# 腿部
for y in range(46, 55):
    for x in range(26, 39):
        if x < 30 or x > 34:
            if 0 <= x < size and 0 <= y < size:
                if pixels[x, y] == (0, 0, 0, 0):
                    pixels[x, y] = DARK_BLUE

# 腰部装饰
for x in range(24, 41):
    if 0 <= x < size and 44 < size:
        pixels[x, 44] = GOLD
        if x % 3 == 0:
            pixels[x, 45] = GOLD

# 莲花座
for y in range(55, 62):
    for x in range(20, 45):
        dx = x - 32
        max_w = 12 - (y - 55)
        if abs(dx) <= max_w:
            if 0 <= x < size and 0 <= y < size:
                if (y - 55) % 2 == 0:
                    pixels[x, y] = PINK
                else:
                    pixels[x, y] = (255, 100, 100, 255)

# 飘带
for x in range(10, 55):
    y = 30 + int(abs(x-32) * 0.2)
    if 0 <= y < size:
        if (x < 22 or x > 42) and (x % 2 == 0):
            if pixels[x, y] == (0, 0, 0, 0):
                pixels[x, y] = RED

# 放大8倍保存
img_big = img.resize((size*8, size*8), Image.NEAREST)
img_big.save('mahakala_pixel.png')
img.save('mahakala_pixel_64.png')

print("大黑天像素图标已生成！")
print("mahakala_pixel.png (512x512)")
print("mahakala_pixel_64.png (64x64)")
