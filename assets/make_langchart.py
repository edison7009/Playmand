"""Generate the README language speed chart SVGs (light/dark x en/zh).

Data: execution time normalized to C, Table 4 of Pereira et al.,
"Energy Efficiency across Programming Languages", SLE 2017.
Run: python assets/make_langchart.py
"""

import math
from pathlib import Path

# (language, time relative to C), as printed in Table 4
TIME = [
    ("C", 1.00),
    ("Rust", 1.04),
    ("C++", 1.56),
    ("Java", 1.89),
    ("Go", 2.83),
    ("C#", 3.14),
    ("JavaScript", 6.52),
    ("TypeScript", 46.20),
    ("Python", 71.90),
    ("Lua", 82.91),
]

TEXT = {
    "en": {
        "title": "Rust runs at C speed",
        "subtitle": "Execution time relative to C · lower is better · log scale",
        "source": "Source: Pereira et al., Energy Efficiency across Programming Languages, SLE 2017 (Table 4)",
    },
    "zh": {
        "title": "Rust 的运行速度接近 C",
        "subtitle": "相对 C 的执行时间 · 越短越好 · 对数刻度",
        "source": "数据来源：Pereira 等，Energy Efficiency across Programming Languages，SLE 2017（表 4）",
    },
}

THEMES = {
    "light": {"surface": "#fcfcfb", "border": "#e1e0d9", "ink": "#0b0b0b", "ink2": "#52514e",
              "muted": "#898781", "grid": "#e1e0d9", "accent": "#eb6834", "rest": "#a8a69e"},
    "dark": {"surface": "#1a1a19", "border": "#2c2c2a", "ink": "#f0efec", "ink2": "#c3c2b7",
             "muted": "#898781", "grid": "#2c2c2a", "accent": "#d95926", "rest": "#5a5955"},
}

FONT = "system-ui, -apple-system, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif"
W, PAD = 880, 32
NAME_COL = 110
BAR_X = PAD + NAME_COL
BAR_MAX = W - BAR_X - PAD - 70
ROW_H, BAR_H, TOP = 30, 16, 128
LO, HI = 0.6, 100.0  # log axis range


def x_of(v):
    return BAR_X + BAR_MAX * math.log(v / LO) / math.log(HI / LO)


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def build(lang, theme):
    c, t = THEMES[theme], TEXT[lang]
    height = TOP + len(TIME) * ROW_H + 64
    out = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{height}" viewBox="0 0 {W} {height}" '
        f'role="img" aria-labelledby="t d" font-family="{FONT}">',
        f'<title id="t">{esc(t["title"])}</title>',
        f'<desc id="d">{esc(t["subtitle"])}. ' + esc("; ".join(f"{n} {v:.2f}x" for n, v in TIME)) + "</desc>",
        f'<rect x="0.5" y="0.5" width="{W - 1}" height="{height - 1}" rx="12" fill="{c["surface"]}" stroke="{c["border"]}"/>',
        f'<text x="{PAD}" y="{PAD + 18}" font-size="20" font-weight="700" fill="{c["ink"]}">{esc(t["title"])}</text>',
        f'<text x="{PAD}" y="{PAD + 42}" font-size="13" fill="{c["ink2"]}">{esc(t["subtitle"])}</text>',
    ]
    grid_top, grid_bottom = TOP - 10, TOP + len(TIME) * ROW_H
    for v in (1, 2, 5, 10, 20, 50, 100):
        x = x_of(v)
        out.append(f'<line x1="{x:.1f}" y1="{grid_top}" x2="{x:.1f}" y2="{grid_bottom}" stroke="{c["grid"]}"/>')
        out.append(f'<text x="{x:.1f}" y="{grid_top - 6}" font-size="11" fill="{c["muted"]}" text-anchor="middle">{v}×</text>')
    for i, (name, v) in enumerate(TIME):
        cy = TOP + i * ROW_H + ROW_H / 2
        rust = name == "Rust"
        fill = c["accent"] if rust else c["rest"]
        ink = c["ink"] if rust else c["ink2"]
        weight = "700" if rust else "400"
        x0, x1, r = BAR_X, x_of(v), 4
        by = cy - BAR_H / 2
        out.append(f'<text x="{BAR_X - 12}" y="{cy + 4:.1f}" font-size="13" font-weight="{weight}" fill="{ink}" text-anchor="end">{name}</text>')
        out.append(
            f'<path d="M{x0} {by:.1f} H{x1 - r:.1f} Q{x1:.1f} {by:.1f} {x1:.1f} {by + r:.1f} '
            f'V{by + BAR_H - r:.1f} Q{x1:.1f} {by + BAR_H:.1f} {x1 - r:.1f} {by + BAR_H:.1f} H{x0} Z" fill="{fill}"/>'
        )
        out.append(f'<text x="{x1 + 8:.1f}" y="{cy + 4:.1f}" font-size="13" font-weight="{weight}" fill="{ink}">{v:.2f}×</text>')
    out.append(f'<text x="{PAD}" y="{height - 24}" font-size="11" fill="{c["muted"]}">{esc(t["source"])}</text>')
    out.append("</svg>")
    return "\n".join(out) + "\n"


def main():
    here = Path(__file__).resolve().parent
    for lang in ("en", "zh"):
        for theme in ("light", "dark"):
            (here / f"langspeed-{lang}-{theme}.svg").write_text(build(lang, theme), encoding="utf-8")


if __name__ == "__main__":
    main()
