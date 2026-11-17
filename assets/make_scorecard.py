"""Generate the README engine scorecard SVGs (light/dark x en/zh).

Run: python assets/make_scorecard.py
Edit SCORES below and re-run to update all four images.
"""

from pathlib import Path

ENGINES = ["Bevy", "Godot", "Unity", "Unreal"]
SCORES = [
    # (english label, chinese label, scores in ENGINES order)
    ("AI coding friendliness", "AI 代码开发友好度", [9.5, 8.5, 8, 7.5]),
    ("ECS / data-driven", "ECS / 数据驱动", [10, 7, 8, 8]),
    ("2D games", "2D 游戏", [8.5, 9.5, 8.5, 7]),
    ("3D games", "3D 游戏", [8, 8, 9, 10]),
    ("Visual editor", "可视化编辑器", [5, 9.5, 10, 10]),
]

TEXT = {
    "en": {
        "title": "Engine scorecard for the AI coding era",
        "subtitle": "Scores out of 10 · subjective comparison by an AI assistant, not a benchmark",
    },
    "zh": {
        "title": "AI 编码时代的引擎评分",
        "subtitle": "满分 10 分 · 来自 AI 助手的主观对比，非基准测试",
    },
}

THEMES = {
    "light": {
        "surface": "#fcfcfb", "border": "#e1e0d9", "ink": "#0b0b0b", "ink2": "#52514e",
        "muted": "#898781", "grid": "#e1e0d9", "accent": "#eb6834", "rest": "#a8a69e",
    },
    "dark": {
        "surface": "#1a1a19", "border": "#2c2c2a", "ink": "#f0efec", "ink2": "#c3c2b7",
        "muted": "#898781", "grid": "#2c2c2a", "accent": "#d95926", "rest": "#5a5955",
    },
}

FONT = "system-ui, -apple-system, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif"
W = 880
PAD = 32
LABEL_COL = 210        # dimension label column
NAME_COL = 64          # engine name column
BAR_X = PAD + LABEL_COL + NAME_COL
BAR_MAX = W - BAR_X - PAD - 44  # leave room for the value label
ROW_H = 20
BAR_H = 12
GROUP_GAP = 18
TOP = 120


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def build(lang, theme):
    c = THEMES[theme]
    t = TEXT[lang]
    group_h = len(ENGINES) * ROW_H
    height = TOP + len(SCORES) * group_h + (len(SCORES) - 1) * GROUP_GAP + PAD
    out = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{height}" '
        f'viewBox="0 0 {W} {height}" role="img" aria-labelledby="t d" font-family="{FONT}">',
        f'<title id="t">{esc(t["title"])}</title>',
        f'<desc id="d">{esc(t["subtitle"])}. '
        + esc("; ".join(f'{row[0]}: ' + ", ".join(f"{e} {v:g}" for e, v in zip(ENGINES, row[2])) for row in SCORES))
        + "</desc>",
        f'<rect x="0.5" y="0.5" width="{W - 1}" height="{height - 1}" rx="12" '
        f'fill="{c["surface"]}" stroke="{c["border"]}"/>',
        f'<text x="{PAD}" y="{PAD + 18}" font-size="20" font-weight="700" fill="{c["ink"]}">{esc(t["title"])}</text>',
        f'<text x="{PAD}" y="{PAD + 42}" font-size="13" fill="{c["ink2"]}">{esc(t["subtitle"])}</text>',
    ]
    # gridlines at 0, 5, 10
    grid_top = TOP - 8
    grid_bottom = height - PAD + 4
    for v in (0, 5, 10):
        x = BAR_X + BAR_MAX * v / 10
        out.append(f'<line x1="{x:.1f}" y1="{grid_top}" x2="{x:.1f}" y2="{grid_bottom}" '
                   f'stroke="{c["grid"]}" stroke-width="1"/>')
        out.append(f'<text x="{x:.1f}" y="{grid_top - 6}" font-size="11" fill="{c["muted"]}" '
                   f'text-anchor="middle">{v}</text>')

    y = TOP
    for en, zh, values in SCORES:
        label = en if lang == "en" else zh
        out.append(f'<text x="{PAD}" y="{y + group_h / 2 + 5:.1f}" font-size="14" font-weight="600" '
                   f'fill="{c["ink"]}">{esc(label)}</text>')
        for i, (engine, v) in enumerate(zip(ENGINES, values)):
            ry = y + i * ROW_H
            is_bevy = engine == "Bevy"
            fill = c["accent"] if is_bevy else c["rest"]
            weight = "700" if is_bevy else "400"
            ink = c["ink"] if is_bevy else c["ink2"]
            bw = BAR_MAX * v / 10
            cy = ry + ROW_H / 2
            out.append(f'<text x="{BAR_X - 10}" y="{cy + 4:.1f}" font-size="12" font-weight="{weight}" '
                       f'fill="{ink}" text-anchor="end">{engine}</text>')
            # rounded data-end, square baseline end
            r = 4
            by = cy - BAR_H / 2
            out.append(
                f'<path d="M{BAR_X} {by:.1f} H{BAR_X + bw - r:.1f} '
                f'Q{BAR_X + bw:.1f} {by:.1f} {BAR_X + bw:.1f} {by + r:.1f} '
                f'V{by + BAR_H - r:.1f} Q{BAR_X + bw:.1f} {by + BAR_H:.1f} {BAR_X + bw - r:.1f} {by + BAR_H:.1f} '
                f'H{BAR_X} Z" fill="{fill}"/>'
            )
            out.append(f'<text x="{BAR_X + bw + 8:.1f}" y="{cy + 4:.1f}" font-size="12" font-weight="{weight}" '
                       f'fill="{ink}">{v:g}</text>')
        y += group_h + GROUP_GAP
        if (en, zh, values) != SCORES[-1]:
            sep = y - GROUP_GAP / 2
            out.append(f'<line x1="{PAD}" y1="{sep:.1f}" x2="{W - PAD}" y2="{sep:.1f}" '
                       f'stroke="{c["grid"]}" stroke-width="1"/>')
    out.append("</svg>")
    return "\n".join(out) + "\n"


def main():
    here = Path(__file__).resolve().parent
    for lang in ("en", "zh"):
        for theme in ("light", "dark"):
            (here / f"scorecard-{lang}-{theme}.svg").write_text(build(lang, theme), encoding="utf-8")


if __name__ == "__main__":
    main()
