#!/usr/bin/env python3
"""Measure the supplied 30 fps Rise samples; never modify input PNGs.

These masks/ROIs are specific to this screenshot set, not a general HUD detector.
Pillow and numpy are already included in this mod's requirements.txt.
"""

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image


def pixels(path):
    with Image.open(path) as image:
        return np.asarray(image.convert("RGB"), dtype=float)


def orange_bbox(image, roi):
    x0, y0, x1, y1 = roi
    data = image[y0:y1, x0:x1]
    red, green, blue = data[..., 0], data[..., 1], data[..., 2]
    mask = (red > 170) & (green > 145) & (green < 205) & (blue < 65)
    y, x = np.where(mask)
    if not len(x):
        raise ValueError(f"No orange pixels in ROI {roi}")
    return [int(x.min()) + x0, int(y.min()) + y0,
            int(x.max()) + x0 + 1, int(y.max()) + y0 + 1]


def slash_fit(image, positive):
    data = image[120:450, 800:1120]
    red, green, blue = data[..., 0], data[..., 1], data[..., 2]
    mask = ((red > 180) & (red - green > 55) & (red - blue > 15)
            if positive else
            (blue > 170) & (blue - green > 55) & (red - green > 25))
    points = []
    for row in range(mask.shape[0]):
        xs = np.flatnonzero(mask[row])
        if len(xs) > 2:
            points.append((row + 120, float(np.median(xs)) + 800))
    if len(points) < 3:
        raise ValueError("Not enough colored beam rows")
    y, x = np.array(points).T
    slope, intercept = np.polyfit(y, x, 1)
    keep = np.abs(x - (slope * y + intercept)) < 10
    slope, intercept = np.polyfit(y[keep], x[keep], 1)
    residual = np.abs(x[keep] - (slope * y[keep] + intercept))
    return {
        "roi_xyxy": [800, 120, 1120, 450],
        "colored_rows": len(points), "retained_rows": int(keep.sum()),
        "dx_per_downward_pixel": round(float(slope), 4),
        "angle_from_downward_vertical_degrees": round(float(np.degrees(np.arctan(slope))), 2),
        "p95_horizontal_fit_residual_pixels": round(float(np.percentile(residual, 95)), 2),
    }


def inspect(source):
    inputs = []
    for group in ["norm_orange", "positive_crit_orange", "negative_crit_orange"]:
        names = [f"{i}.png" for i in range(1, 4 if group == "norm_orange" else 7)]
        if group != "negative_crit_orange":
            names.append("before_go.png")
        for name in names:
            path = source / group / name
            with Image.open(path) as image:
                dimensions = list(image.size)
            if dimensions != [1919, 1080]:
                raise ValueError(f"Unexpected dimensions for {path}: {dimensions}")
            inputs.append({"path": f"{group}/{name}", "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                           "dimensions": dimensions})
    bounce = []
    for group, settled, roi in [("norm_orange", "3.png", [954, 548, 1008, 590]),
                                ("positive_crit_orange", "6.png", [940, 520, 991, 561])]:
        before = orange_bbox(pixels(source / group / settled), roi)
        after = orange_bbox(pixels(source / group / "before_go.png"), roi)
        bounce.append({"group": group, "settled_sample": settled, "exit_sample": "before_go.png",
                       "roi_xyxy": roi, "settled_orange_bbox_xyxy": before, "exit_orange_bbox_xyxy": after,
                       "top_delta_pixels": after[1] - before[1], "bottom_delta_pixels": after[3] - before[3]})
    beams = []
    for group in ["positive_crit_orange", "negative_crit_orange"]:
        for frame in [3, 4, 5]:
            result = slash_fit(pixels(source / group / f"{frame}.png"), group.startswith("positive"))
            beams.append({"path": f"{group}/{frame}.png", **result})
    outline = pixels(source / "norm_orange/before_go.png")
    return {
        "schema_version": 1, "inspection_date": "2026-10-08", "kind": "user screenshot research; runtime unchanged",
        "source": "User-supplied hit_screenshots directory; paths below are relative to it",
        "reported_capture_fps": 30, "inputs": inputs,
        "measurement_limits": [
            "Orange ROIs include antialiasing and possible reticle interference; bboxes are approximate fill extents, not full glyph metrics.",
            "before_go is a user-labeled exit transition sample, not necessarily adjacent to the previous numbered PNG.",
            "Slash fits use colored glow medians above the hit, not exact engine geometry; bloom affects the positive estimate.",
            "Only one positive and one negative crit are supplied; the random angle range/distribution is unverified.",
            "No full exit sequence or hold timestamps are supplied; total lifetime and runtime clip tick rate are unmeasured.",
            "Orange is the only measured digit color; shared motion for other colors is user direction.",
            "Local Rise size-mod activation and effective render scaling are unverified.",
        ],
        "orange_mask": "R>170, 145<G<205, B<65; exclusive maximum coordinates",
        "exit_bounce": bounce, "slash_fits": beams,
        "outline_row_profile": {
            "path": "norm_orange/before_go.png", "y": 566,
            "pixels": [{"x": x, "rgb": outline[566, x].astype(int).tolist()}
                       for x in range(996, 1006)],
            "interpretation": "Gold/antialiased edge at x996-998, near-black band x999-1003, then background transition. Roughly 4-5 px dark edge at this row.",
        },
        "visual_observations": {
            "normal": "38; common blue-white burst in 1/2, orange digits with thick black edge in 3 and before_go; no long crit slash.",
            "positive": "48; red glow with bright pale core in 3/4, thin fading slash in 5, beam gone in 6; digits stay orange.",
            "negative": "58; purple/violet glow with bright pale core in 3/4, thin fading slash in 5, beam gone in 6; older nearby 78 is a separate event.",
            "slash_geometry": "Long tapered streak through the impact, behind digits, extending much farther above than below; longest samples are clipped by top screen edge.",
        },
        "mhgu_design_direction": {
            "omit": "Shared blue-white hit flash in first two samples; MHGU already supplies hit effects. Preserve digit entrance.",
            "digits": "Broad rendered digits with thick nearly solid black outline and lightly softened outer edge; calibrate full outlined glyphs.",
            "motion": "Grow/shrink/settle, hold, small +4 reference-unit downward kick at exit, then upward alpha fade; time based, not one rendered frame at every FPS.",
            "affinity": "Red positive slash, purple negative slash, none for noncrit; digit palette remains independent of affinity.",
            "angle": "User revised target to approximately +/-15 degrees. Proposed slope s in [-1/4,+1/4], approximately +/-14.04 degrees. Choose once per hit; upward direction (s,-1) needs no trig.",
            "other_colors": "Share animation and outline behavior, per user direction; retain existing MHGU damage-color classification.",
        },
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="Directory containing the three supplied screenshot groups")
    parser.add_argument("output", type=Path, help="Destination JSON; source PNGs are untouched")
    args = parser.parse_args()
    result = inspect(args.source)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(f"Recorded {len(result['inputs'])} screenshots, two exit comparisons and six beam fits")


if __name__ == "__main__":
    main()
