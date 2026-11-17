#!/usr/bin/env python3
"""Original 30-second score for the Playmand promo.

No samples and no third-party melody. Deterministic from seed 7, so
re-running this file reproduces assets/music/promo.wav exactly.
120 BPM, A minor, arranged to the same shot cuts as the film
(0 / 4 / 8 / 16 / 24 seconds).
"""

from __future__ import annotations

import sys
import wave
from pathlib import Path

import numpy as np
from scipy.signal import lfilter

SR = 48_000
SECONDS = 30.0
N = int(SR * SECONDS)
RNG = np.random.default_rng(7)


def env_adsr(n: int, attack: float, decay: float, sustain: float, release: float) -> np.ndarray:
    t = np.arange(n) / SR
    attack_n = max(1, int(attack * SR))
    out = np.ones(n)
    out[:attack_n] = np.linspace(0.0, 1.0, attack_n, endpoint=False)
    after = t[attack_n:] - attack
    out[attack_n:] = sustain + (1.0 - sustain) * np.exp(-after / max(decay, 1e-4))
    rel = int(release * SR)
    if rel > 1:
        out[-rel:] *= np.linspace(1.0, 0.0, rel)
    return out


def place(bus: np.ndarray, start: float, clip: np.ndarray, gain: float = 1.0) -> None:
    i = int(start * SR)
    j = min(N, i + clip.shape[-1])
    if i >= N or j <= i:
        return
    span = j - i
    if bus.ndim == 1:
        bus[i:j] += clip[:span] * gain
    elif clip.ndim == 1:
        bus[:, i:j] += clip[:span] * gain
    else:
        bus[:, i:j] += clip[:, :span] * gain


def tone(freq: float, dur: float, gain: float = 1.0, decay: float = 0.18) -> np.ndarray:
    n = int(dur * SR)
    t = np.arange(n) / SR
    wave_s = (
        np.sin(2 * np.pi * freq * t)
        + 0.18 * np.sin(2 * np.pi * freq * 2 * t)
        + 0.05 * np.sin(2 * np.pi * freq * 3 * t)
    )
    return wave_s * env_adsr(n, 0.008, decay, 0.0, 0.04) * gain


def kick() -> np.ndarray:
    dur = 0.22
    n = int(dur * SR)
    t = np.arange(n) / SR
    freq = 165 * np.exp(-t * 26) + 46
    phase = 2 * np.pi * np.cumsum(freq) / SR
    body = np.sin(phase) * np.exp(-t * 10)
    click = RNG.normal(0.0, 1.0, n) * np.exp(-t * 90) * 0.22
    return (body + click) * 0.95


def snare() -> np.ndarray:
    dur = 0.18
    n = int(dur * SR)
    t = np.arange(n) / SR
    noise = RNG.normal(0.0, 1.0, n)
    noise = lfilter([1.0], [1.0, -0.4], noise)
    body = np.sin(2 * np.pi * 196 * t) * np.exp(-t * 28) * 0.35
    return (noise * 0.55 + body) * np.exp(-t * 16)


def hat(open_hat: bool = False) -> np.ndarray:
    dur = 0.09 if open_hat else 0.035
    n = int(dur * SR)
    t = np.arange(n) / SR
    noise = RNG.normal(0.0, 1.0, n)
    noise = np.diff(noise, prepend=0)
    decay = 11 if open_hat else 48
    return noise * np.exp(-t * decay) * (0.34 if open_hat else 0.22)


def bass_note(freq: float) -> np.ndarray:
    return tone(freq, 0.24, gain=0.72, decay=0.16) + 0.25 * tone(freq / 2, 0.24, gain=0.8, decay=0.2)


def pad_chord(freqs: list[float], dur: float) -> np.ndarray:
    n = int(dur * SR)
    t = np.arange(n) / SR
    left = np.zeros(n)
    right = np.zeros(n)
    for index, freq in enumerate(freqs):
        detune = 1.004 if index % 2 == 0 else 0.996
        wave_s = np.sin(2 * np.pi * freq * t) + 0.35 * np.sin(2 * np.pi * freq * detune * t)
        left += wave_s * (0.5 if index != 1 else 0.2)
        right += np.sin(2 * np.pi * freq * (2 - detune) * t)
    bed = np.stack([left, right]) / max(len(freqs), 1)
    attack = int(0.08 * SR)
    bed[:, :attack] *= np.linspace(0.0, 1.0, attack)
    release = int(0.2 * SR)
    bed[:, -release:] *= np.linspace(1.0, 0.0, release)
    return bed * 0.16


def riser(dur: float) -> np.ndarray:
    n = int(dur * SR)
    t = np.arange(n) / SR
    noise = RNG.normal(0.0, 1.0, n)
    noise = np.diff(noise, prepend=0)
    center = 400 + 2800 * (t / dur) ** 2
    tone_s = np.sin(2 * np.pi * center * t)
    amp = (t / dur) ** 1.6
    mono = (noise * 0.35 + tone_s * 0.25) * amp
    return np.stack([mono, np.roll(mono, 180)]) * 0.45


def impact() -> np.ndarray:
    n = int(0.6 * SR)
    t = np.arange(n) / SR
    noise = RNG.normal(0.0, 1.0, n) * np.exp(-t * 7)
    sub = np.sin(2 * np.pi * 48 * t) * np.exp(-t * 4)
    mono = noise * 0.45 + sub * 0.7
    return np.stack([mono, mono])


def chord_at(t: float) -> tuple[float, list[float]]:
    """Return (bass Hz, pad chord) for the arrangement."""
    a2, c3, e3, f2, g2, b2, d3 = 110.0, 130.81, 164.81, 87.31, 98.0, 123.47, 146.83
    if t < 8.0:
        return a2, [220.0, 261.63, 329.63]
    if t < 12.0:
        return f2, [174.61, 220.0, 261.63]
    if t < 16.0:
        return c3, [261.63, 329.63, 392.0]
    if t < 20.0:
        return a2, [220.0, 261.63, 329.63]
    if t < 24.0:
        return g2, [196.0, b2 * 2, d3 * 2]
    return a2 / 2, [220.0, 261.63, 329.63, 440.0]


def main() -> None:
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parents[1] / "assets" / "music" / "promo.wav"
    drums = np.zeros(N)
    bass = np.zeros(N)
    music = np.zeros((2, N))

    kick_s = kick()
    snare_s = snare()
    hat_s = hat(False)
    open_s = hat(True)

    # Intro downbeats, then four-on-the-floor, then a quieter ending.
    for bar_start in (0.0, 2.0):
        place(drums, bar_start, kick_s, 0.75)
    for step in np.arange(4.0, 24.0, 0.5):
        place(drums, float(step), kick_s, 0.9)
    for step in (24.0, 26.0, 28.0):
        place(drums, step, kick_s, 0.55)
    for step in np.arange(4.0, 24.0, 2.0):
        place(drums, float(step) + 1.0, snare_s, 0.72)
    for step in np.arange(4.0, 24.0, 0.25):
        open_hat = abs((step % 2.0) - 1.5) < 0.01
        place(drums, float(step), open_s if open_hat else hat_s, 0.8)

    # Bass: root on each beat, a fifth answering on the offbeat.
    for step in np.arange(4.0, 24.0, 0.5):
        root, _ = chord_at(float(step))
        freq = root if (step * 2) % 2 < 0.1 or int(step * 2) % 4 == 0 else root * 1.5
        if int(step * 2) % 4 == 3:
            freq = root * 2
        place(bass, float(step), bass_note(freq), 0.85)
    place(bass, 24.0, bass_note(55.0), 0.7)
    place(bass, 26.5, bass_note(82.41), 0.45)

    # Motif: the same five-note shape opens the film and returns on the end card.
    motif = [440.0, 523.25, 659.25, 880.0, 783.99, 659.25, 523.25, 440.0]
    for index, freq in enumerate(motif):
        note = tone(freq, 0.42, gain=0.34, decay=0.2)
        start = 0.12 + index * 0.46
        stereo = np.stack([note, note * 0.85])
        place(music, start, stereo, 1.0)
        place(music, 24.15 + index * 0.62, tone(freq, 0.7, gain=0.28, decay=0.35), 0.9)

    # Arpeggio locked to the chord of the current bar.
    step = 4.0
    while step < 23.8:
        _, chord = chord_at(step)
        cycle = [chord[0], chord[1], chord[2], chord[1] * 2, chord[2], chord[0] * 2]
        freq = cycle[int(step * 4) % len(cycle)]
        note = tone(freq, 0.16, gain=0.16, decay=0.08)
        delayed = np.zeros_like(note)
        shift = 280
        delayed[shift:] = note[:-shift]
        place(music, step, np.stack([note, delayed]), 1.0)
        step += 0.125

    for bar in (0.0, 4.0, 8.0, 12.0, 16.0, 20.0, 24.0):
        _, chord = chord_at(bar + 0.01)
        dur = 6.2 if bar == 24.0 else 4.15
        place(music, bar, pad_chord(chord, dur), 1.0)

    for hit, dur in ((3.15, 0.85), (7.2, 0.8), (14.7, 1.3), (22.9, 1.1)):
        place(music, hit, riser(dur), 0.8 if hit != 14.7 else 1.0)
    place(music, 16.0, impact(), 0.85)
    place(music, 24.0, impact(), 0.55)

    # Sidechain the musical bed to the kick so the beat reads.
    kick_env = np.zeros(N)
    for step in list(np.arange(4.0, 24.0, 0.5)) + [0.0, 2.0, 24.0, 26.0, 28.0]:
        i = int(step * SR)
        n = int(0.18 * SR)
        j = min(N, i + n)
        if i >= N:
            continue
        kick_env[i:j] = np.maximum(kick_env[i:j], np.exp(-np.arange(j - i) / (0.045 * SR)))
    music *= 1.0 - 0.55 * kick_env

    mix_l = drums * 0.82 + bass * 0.78 + music[0]
    mix_r = drums * 0.82 + bass * 0.78 + music[1]
    mix = np.stack([mix_l, mix_r])
    mix = lfilter([0.18, 0.18], [1.0, -0.64], mix, axis=1)
    fade_in = int(0.03 * SR)
    fade_out = int(0.45 * SR)
    mix[:, :fade_in] *= np.linspace(0.0, 1.0, fade_in)
    mix[:, -fade_out:] *= np.linspace(1.0, 0.0, fade_out)
    peak = np.max(np.abs(mix))
    mix = np.tanh(mix / peak * 1.35)
    mix = mix / np.max(np.abs(mix)) * 0.89

    out.parent.mkdir(parents=True, exist_ok=True)
    pcm = (mix.T * 32767.0).astype(np.int16)
    with wave.open(str(out), "wb") as handle:
        handle.setnchannels(2)
        handle.setsampwidth(2)
        handle.setframerate(SR)
        handle.writeframes(pcm.tobytes())

    rms = [float(np.sqrt(np.mean(mix[:, int(s * SR): int((s + 1) * SR)] ** 2))) for s in range(int(SECONDS))]
    print(f"wrote {out} ({out.stat().st_size / 1024:.0f} KiB)")
    print("rms/sec", " ".join(f"{v:.3f}" for v in rms))


if __name__ == "__main__":
    main()
