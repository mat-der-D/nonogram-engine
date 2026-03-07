# Image → Dot Grid Specification

Converts a raster image into a binary dot grid suitable for use as a nonogram source.

---

## Overview

The user loads a single image file. The image is resized once on load, then a dot grid is generated in real time as the user adjusts parameters. The UI shows the resized source image and the generated dot grid side by side.

---

## Image Loading and Preprocessing

### Alpha Compositing

If the image has an alpha channel, it is composited onto a **white background** before any further processing.

```
result = rgb * (alpha / 255) + white * (1 - alpha / 255)
```

This step is applied wherever the raw image data is used: both for the source preview and for dot generation.

### Resize (once on load)

The image is resized to fit within a **384 × 384 px** bounding box, using area interpolation. The aspect ratio is preserved. Images smaller than 384 px on all edges are not scaled up.

The resized image is stored and reused for all subsequent dot generation without re-loading from disk.

---

## Processing Pipeline

Executed in this order each time any parameter changes:

```
grayscale
→ gaussian blur          (smooth_strength)
→ canny edge detection   (edge_strength)
→ edge merge             (edge_strength)
→ cell-mean downsampling (grid_width, grid_height)
→ threshold              (threshold)
→ noise removal          (noise_removal)
```

### 1. Grayscale Conversion

The image (already alpha-composited if needed) is converted to grayscale using standard luminance weighting.

### 2. Gaussian Blur

Reduces fine detail and noise before edge detection and downsampling.

- **Applied when:** `smooth_strength > 0`
- **Sigma:** `smooth_strength`
- **Kernel size:** smallest odd integer ≥ `smooth_strength × 4 + 1`

### 3. Canny Edge Detection

Detects edges in the blurred grayscale image.

- **Applied when:** `edge_strength > 0`
- **Low threshold:** 50
- **High threshold:** 150
- Input: the blurred grayscale image

### 4. Edge Merge

Detected edges are blended back into the grayscale image by darkening pixels at edge locations. This causes edges to contribute filled dots after thresholding.

```
pixel = clamp(gray - edge_map * edge_strength, 0, 255)
```

`edge_map` is the binary edge image (0 or 255 per pixel).

### 5. Cell-Mean Downsampling

The grayscale image is divided into a `grid_width × grid_height` grid of rectangular cells. Each cell's mean pixel brightness is computed and becomes that cell's value, producing a `grid_height × grid_width` floating-point grid.

Pixels that do not fit evenly into cells (remainder at the bottom and right edges) are excluded.

### 6. Threshold

Each cell is classified as **filled** (1) or **empty** (0) based on its mean brightness.

```
cell = 1  if mean < threshold
cell = 0  otherwise
```

Dark cells become filled dots. Bright cells become empty.

### 7. Noise Removal

Small isolated filled regions are removed from the binary grid.

- **Applied when:** `noise_removal > 0`
- **Connectivity:** 4-connected
- Connected components of filled cells with area **less than** `noise_removal` cells are cleared to empty.
- The background (empty cell regions) is not affected.

---

## User Parameters

### Grid Settings

| Parameter    | Range | Default | Description                        |
|--------------|----|------|------------------------------------|
| `grid_width`  | 5–50 | 20 | Number of dot columns              |
| `grid_height` | 5–50 | 20 | Number of dot rows                 |

### Image Processing

| Parameter        | Range | Default | Description                                               |
|------------------|-------|---------|-----------------------------------------------------------|
| `smooth_strength` | 0–5   | 1.0     | Gaussian blur sigma. 0 disables blur.                     |
| `threshold`       | 0–255 | 128     | Brightness cutoff. Lower → fewer filled dots.             |
| `edge_strength`   | 0–1   | 0.3     | How strongly edges darken the image. 0 disables edge detection. |
| `noise_removal`   | 0–20  | 0       | Minimum filled component size in cells. 0 disables removal. |

---

## UI Layout

```
[ Source preview ]  [ Dot grid preview ]

────────────────────────────────────────

Grid Width      [slider]
Grid Height     [slider]
Smooth Strength [slider]
Threshold       [slider]
Edge Strength   [slider]
Noise Removal   [slider]

[ Load Image ]
```

The dot grid preview updates immediately on any slider change. The source preview updates only when a new image is loaded.
