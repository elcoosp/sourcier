# ADR 0002: Logo format — traced SVG from raster original

**Status:** Accepted
**Date:** 2026-11-07

## Context

The repository ships exactly one logo asset: `docs/logo.png`
(1264×1264 RGBA). No vector original exists anywhere in the repository
(searched `assets/`, `public/`, `static/`, `brand/`, `design/`). Shipping a
raster logo in a Starlight header risks pixelation when scaled beyond its
native resolution.

## Decision

1. **Trace the raster to SVG** with `vtracer` (color mode, stacked
   hierarchy, spline curves, `filter_speckle=12`, `color_precision=7`,
   `layer_difference=32`). Intermediate traces with heavier simplification
   (53 KiB / 12 KiB) were rejected because their color clustering merged the
   indigo top layer into the teal middle layer, destroying the logo
   identity.
2. **Preserve `viewBox`.** The tracer emits only `width`/`height`, so
   `viewBox="0 0 1264 1264"` was added explicitly — without it the SVG
   cannot scale and would be cropped or blurry.
3. **Optimize with SVGO** (`--multipass`, `removeViewBox: false`,
   408 KiB → 249 KiB) and commit the optimized asset at
   `src/assets/logo.svg`.
4. **Favicon**: `public/favicon.svg` is a separate trace rendered from a
   64 px LANCZOS downscale of the original (22 KiB, `viewBox` preserved), so
   the 16–32 px favicon renders from fewer, larger paths.
5. **Single logo for both themes**: the artwork sits on a transparent
   background with mid-tone colors and reads correctly on both light and
   dark headers, so `logo: { src: ... }` is used instead of light/dark
   variants. `replacesTitle` is NOT set because the logo contains no
   wordmark — the text title stays visible.

## Astrogotchas considered

- Astro's experimental `svgOptimizer` was **not** enabled: section 6.2 of
  the project constraints forbids experimental Astro flags in the docs
  config, and the committed asset is already SVGO-optimized (step 3), which
  covers the same need at commit time.
- The logo was confirmed sharp from rendered screenshots at 2× in both
  themes during the Phase 7 eyeball pass, not assumed from the pipeline.

## Consequences

- `src/assets/logo.svg` (249 KiB) and `public/favicon.svg` (22 KiB) are the
  only brand assets; both scale losslessly.
- If the project ever publishes a true vector original, replace both files
  and delete the tracing step from the pipeline.
