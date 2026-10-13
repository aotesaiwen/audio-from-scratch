"""Praat's numbers for one recording, to compare with lesson 14's report.

Uses the same Praat calls as a typical voice-disorder feature script
(To Pitch, To PointProcess (periodic, cc), To Harmonicity (cc), and the
0.0001 / 0.02 / 1.3 / 1.6 limits).

    pip install praat-parselmouth
    python tools/praat_reference.py path/to/sustained_a.wav [f0min f0max]

Then:
    cargo run --example lesson14 -- path/to/sustained_a.wav
"""

import sys

import parselmouth
from parselmouth.praat import call

path = sys.argv[1]
f0min, f0max = (float(sys.argv[2]), float(sys.argv[3])) if len(sys.argv) > 3 else (75.0, 600.0)

sound = parselmouth.Sound(path).convert_to_mono()
pitch = call(sound, "To Pitch", 0.0, f0min, f0max)
points = call(sound, "To PointProcess (periodic, cc)", f0min, f0max)
harmonicity = call(sound, "To Harmonicity (cc)", 0.01, f0min, 0.1, 1.0)
limits = (0, 0, 0.0001, 0.02, 1.3)


def jitter(kind):
    return call(points, f"Get jitter ({kind})", *limits)


def shimmer(kind):
    return call([sound, points], f"Get shimmer ({kind})", *limits, 1.6)


rows = [
    ("Mean F0:", call(pitch, "Get mean", 0, 0, "Hertz"), "Hz"),
    ("Std F0:", call(pitch, "Get standard deviation", 0, 0, "Hertz"), "Hz"),
    ("Jitter (local):", 100 * jitter("local"), "%"),
    ("Jitter (local, abs):", 1e6 * jitter("local, absolute"), "µs"),
    ("Jitter (rap):", 100 * jitter("rap"), "%"),
    ("Shimmer (local):", 100 * shimmer("local"), "%"),
    ("Shimmer (local, dB):", shimmer("local_dB"), "dB"),
    ("Shimmer (apq3):", 100 * shimmer("apq3"), "%"),
    ("HNR:", call(harmonicity, "Get mean", 0, 0), "dB"),
]
print(path)
for name, value, unit in rows:
    print(f"{name:<24} {value:8.3f} {unit}")
