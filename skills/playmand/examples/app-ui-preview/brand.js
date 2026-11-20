// A card colour for each brand, read from its own icon, so new entries need
// no hand-kept palette. Same rule as scripts/brand_colour.py; port it into the
// app as a small module once the look is agreed.
//
//   const [from, to, neutral, glyph] = await brandColour('icons/x.svg');
//
// from/to: the card's gradient stops (rgb(), no percent signs).
// neutral: black, white or grey logo: the card is slate.
// glyph:   a one-colour mark on transparency: draw it white on its own colour.

function hslToRgb(h, s, l) {
  const k = (n) => (n + h / 30) % 12;
  const a = s * Math.min(l, 1 - l);
  const f = (n) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
  return [f(0), f(8), f(4)];
}
const css = ([r, g, b]) => `rgb(${Math.round(r * 255)} ${Math.round(g * 255)} ${Math.round(b * 255)})`;
function contrastWithWhite(h, s, l) {
  const lin = (c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const [r, g, b] = hslToRgb(h, s, l).map(lin);
  return 1.05 / (0.2126 * r + 0.7152 * g + 0.0722 * b + 0.05);
}
function rgbToHsl(r, g, b) {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b), l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min, s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  const h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [h * 60, s, l];
}

export const NEUTRAL = [css(hslToRgb(220, 0.09, 0.3)), css(hslToRgb(220, 0.1, 0.22)), true, false];

export function brandFromPixels(data) {
  const bins = Array.from({ length: 36 }, () => ({ w: 0, h: 0, s: 0, l: 0 }));
  const hues = [];
  let solid = 0, coloured = 0, dark = 0;
  for (let i = 0; i < data.length; i += 4) {
    if (data[i + 3] < 140) continue;
    solid++;
    const [h, s, l] = rgbToHsl(data[i], data[i + 1], data[i + 2]);
    if (s < 0.25 && l < 0.3) dark++;
    if (s < 0.25 || l < 0.12 || l > 0.92) continue;
    coloured++;
    hues.push(h);
    const bin = bins[Math.floor(h / 10) % 36], w = s * (1 - Math.abs(l - 0.5));
    bin.w += w; bin.h += h * w; bin.s += s * w; bin.l += l * w;
  }
  const thin = solid < (data.length / 4) * 0.6;
  if (!solid || coloured / solid < 0.08) return [NEUTRAL[0], NEUTRAL[1], true, thin && solid > 0 && dark / solid > 0.5];
  // Pool each bin with its neighbours so a gradient icon picks its centre.
  let best = 0, bestW = -1;
  for (let i = 0; i < 36; i++) {
    const w = bins[(i + 35) % 36].w * 0.5 + bins[i].w + bins[(i + 1) % 36].w * 0.5;
    if (w > bestW) { bestW = w; best = i; }
  }
  const bin = bins[best];
  if (!bin.w) return NEUTRAL;
  const h = bin.h / bin.w;
  const near = hues.filter((x) => Math.min(Math.abs(x - h), 360 - Math.abs(x - h)) < 25).length;
  const s = Math.min(Math.max(bin.s / bin.w, 0.5), 0.82);
  // Yellows and greens read lighter, so they start darker; then darken until
  // white text has at least 3:1 contrast (WCAG, large bold text).
  let l = Math.min(Math.max(bin.l / bin.w, 0.36), 0.5) - (h > 40 && h < 170 ? 0.07 : 0);
  while (contrastWithWhite(h, s, l) < 3 && l > 0.2) l -= 0.02;
  return [css(hslToRgb(h, s, l)), css(hslToRgb((h + 4) % 360, Math.min(s + 0.04, 0.9), l - 0.08)), false, thin && near / solid > 0.75];
}

const cache = new Map();
export function brandColour(src) {
  if (!cache.has(src)) {
    cache.set(src, new Promise((done) => {
      const img = new Image();
      img.onload = () => {
        try {
          const canvas = document.createElement('canvas');
          canvas.width = canvas.height = 40;
          const g = canvas.getContext('2d', { willReadFrequently: true });
          g.drawImage(img, 0, 0, 40, 40);
          done(brandFromPixels(g.getImageData(0, 0, 40, 40).data));
        } catch { done(NEUTRAL); }
      };
      img.onerror = () => done(NEUTRAL);
      img.src = src;
    }));
  }
  return cache.get(src);
}
