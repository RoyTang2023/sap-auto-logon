#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""生成应用图标：蓝色底 + 白色树形 + 登录箭头。

图标是自造图形，不引用其他项目的资源；改动后重跑本脚本即可重建 assets/icon.ico。
"""

from __future__ import annotations

import math
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 256
OUT = Path(__file__).resolve().parent.parent / "assets" / "icon.ico"

BLUE_TOP = (10, 110, 209)
BLUE_BOTTOM = (8, 74, 138)
WHITE = (255, 255, 255, 255)


def rounded_mask(size: int, radius: int) -> Image.Image:
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, size - 1, size - 1), radius, fill=255)
    return mask


def gradient(size: int) -> Image.Image:
    image = Image.new("RGBA", (size, size))
    draw = ImageDraw.Draw(image)
    for y in range(size):
        t = y / (size - 1)
        color = tuple(int(BLUE_TOP[i] + (BLUE_BOTTOM[i] - BLUE_TOP[i]) * t) for i in range(3))
        draw.line([(0, y), (size, y)], fill=color + (255,))
    return image


def draw_tree(draw: ImageDraw.ImageDraw) -> None:
    """左侧三个节点用一条主干串起来，每个节点向右一段引线。"""
    trunk_x = 78
    stroke = 9
    draw.line([(trunk_x, 66), (trunk_x, 196)], fill=WHITE, width=stroke)
    for index, y in enumerate((66, 128, 196)):
        radius = 15 if index != 1 else 18
        draw.ellipse(
            (trunk_x - radius, y - radius, trunk_x + radius, y + radius),
            fill=WHITE,
        )
        draw.line([(trunk_x + radius, y), (122, y)], fill=WHITE, width=stroke)

    # 中间那条引线接一个向右的箭头，表示「从这里登录进去」。
    draw.rectangle((112, 128 - 14, 128, 128 + 14), fill=WHITE)
    draw.polygon([(128, 128 - 22), (152, 128), (128, 128 + 22)], fill=WHITE)


def draw_window(draw: ImageDraw.ImageDraw) -> None:
    """右侧的目标系统：白边圆角窗口，中间一条横线当标题栏。"""
    draw.rounded_rectangle((160, 84, 228, 172), radius=14, outline=WHITE, width=9)
    draw.line([(168, 112), (220, 112)], fill=WHITE, width=7)


def main() -> None:
    image = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    image.paste(gradient(SIZE), (0, 0), rounded_mask(SIZE, 56))
    draw = ImageDraw.Draw(image)
    draw_tree(draw)
    draw_window(draw)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    image.save(OUT, format="ICO", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    # 托盘图标单独一张 32x32 PNG：tray-icon 要 RGBA，直接从 ICO 解帧尺寸不可控。
    tray = OUT.with_name("tray-32.png")
    image.resize((32, 32), Image.LANCZOS).save(tray, format="PNG")
    print(f"icon written: {OUT}")
    print(f"tray icon written: {tray}")


if __name__ == "__main__":
    main()
