# Visual QA Report — Sourcier documentation

Final QA run against the production build (`astro build` → `astro preview` → `node scripts/qa-screenshots.mjs`).

**Automated result:** `QA PASSED — 18 routes × 2 viewports × 2 themes` (72 screenshots, zero failures). `pnpm build`, `pnpm check` and `pnpm lint` all exited 0 with zero warnings.

- Viewports: desktop 1440×900, mobile 390×844
- Themes: light and dark, forced via `localStorage["starlight-theme"]` and verified against `document.documentElement.dataset.theme`
- Automated probes per route/theme/viewport: exactly one `<h1>`, main text length ≥ 200, no horizontal overflow, no console errors, no page errors, no HTTP ≥ 400 responses, no broken `<img>`
- The `/404` route was screenshotted separately once (desktop, light) plus the four standard variants exist for it in `.qa/` from earlier runs

## Per-route eyeball results

Legend: ✅ pass after fix (see notes), ✅ pass.

| Route | Desktop light | Desktop dark | Mobile light | Mobile dark | Notes |
| --- | --- | --- | --- | --- | --- |
| `/` (splash landing) | ✅ | ✅ | ✅ | ✅ | Logo sharp in header and hero in both themes; accent buttons are brand green, not Starlight blue; CardGrid/LinkCards render with icons and spacing |
| `/tutorials/getting-started/` | ✅ | ✅ | ✅ | ✅ | `<Steps>` numbered circles, aside, Rust code blocks syntax-highlighted with multiple token colors, right-rail TOC highlights current heading, pagination correct (Overview ↔ Installation) |
| `/guides/installation/` | ✅ | ✅ | ✅ | ✅ | `<Tabs>` (cargo add / Cargo.toml) render; note + caution asides styled; code blocks scroll horizontally without page overflow |
| `/guides/registering-files/` | ✅ | ✅ | ✅ | ✅ | Id-type table fits; asides render |
| `/guides/working-with-positions/` | ✅ | ✅ | ✅ | ✅ | Tables and const-context aside render |
| `/guides/navigating-source/` | ✅ | ✅ | ✅ | ✅ | Limits table renders; caution aside visible |
| `/guides/usage-statistics/` | ✅ | ✅ | ✅ | ✅ | Snapshot-fields table and multi-thread example render |
| `/guides/handling-errors/` | ✅ | ✅ | ✅ | ✅ | Variant table and text block of display strings render |
| `/guides/tuning-performance/` | ✅ | ✅ | ✅ | ✅ | Benchmark table fits and scrolls correctly on mobile |
| `/reference/api/` | ✅ | ✅ | ✅ | ✅ | **Fixed twice.** (1) Wide signature tables overflowed the viewport on mobile — fixed with `display:block; max-width:100%; overflow-x:auto` on tables (the first attempt used `max-width:fit-content`, which still widened the page). (2) `:badge[...]` directives rendered as raw text inside table cells (the heading-badges plugin only processes headings) — replaced with `<Badge text="..." size="small" />` components; heading badges keep the directive form. A third defect was found by the automated probe: the inline heading `FeedbackSnapshot :badge[rt-feedback]` exceeded the viewport on mobile because the anchor-link reserved `padding-right` cannot wrap — fixed by dropping that padding below 640 px |
| `/reference/position-encoding/` | ✅ | ✅ | ✅ | ✅ | Bit-layout tables, limits table and caution aside render in dark mode |
| `/reference/feature-flags/` | ✅ | ✅ | ✅ | ✅ | TOML snippets and tables render |
| `/reference/errors/` | ✅ | ✅ | ✅ | ✅ | Long display strings inside the variant table wrap or scroll correctly |
| `/reference/changelog/` | ✅ | ✅ | ✅ | ✅ | Release sections and note render |
| `/explanation/why-sourcier/` | ✅ | ✅ | ✅ | ✅ | Note aside, bullets, LinkCards, pagination all correct in dark mode |
| `/explanation/two-phase-lifecycle/` | ✅ | ✅ | ✅ | ✅ | Code block inside explanation renders with highlighting |
| `/explanation/performance-design/` | ✅ | ✅ | ✅ | ✅ | Two benchmark tables render; benchmark numbers spot-checked against PERFORMANCE.md |
| `/explanation/safety-and-stability/` | ✅ | ✅ | ✅ | ✅ | Test-layer table renders |
| `/404` (extra screenshot) | ✅ | — | — | — | **Fixed:** the custom 404 entry is a draft (so Starlight's built-in 404 route picks it up without generating a conflicting dynamic route), which made Starlight render its draft banner in production; hidden on that page only via frontmatter `head` style. Entry-point LinkCards render |

## Checklist outcomes

- [x] **Logo sharp** in both themes at header size and in the hero (vector SVG with `viewBox`, traced from the source PNG; inspected at 2× — no blur, halo or clipping)
- [x] **Brand accent** visible on links, active sidebar item, focus rings and buttons — spring green derived from the logo glow, not Starlight default blue; WCAG AA checked programmatically (see `src/styles/custom.css` header comment)
- [x] **Code blocks** syntax-highlighted (Shiki/Expressive Code, multiple token colors), copy button present, horizontal scroll contained, `ansi` output blocks render
- [x] **Tables** fit or scroll inside the content column; no cut-off page overflow
- [x] **Asides, CardGrid, Tabs, Steps, LinkCards** render with correct icons, spacing, borders
- [x] **Sidebar** active state correct; groups expand; no duplicated or missing items
- [x] **On-page TOC** present on long pages, highlights current heading; no duplicate entries (verified in built HTML)
- [x] **Pagination** prev/next labels match sidebar order on every checked page
- [x] **Mobile hamburger** opens the sidebar drawer (`button.sl-menu-button` → `starlight__sidebar` popover), close button appears (`scripts/manual-checks.mjs`)
- [x] **Search modal** opens with Ctrl+K, Pagefind returns 23 relevant results with titles and excerpts for "finalize" (`scripts/manual-checks.mjs`)
- [x] **Dark mode**: no pure-white panels, no invisible text, code block background distinct from page background
- [x] **Typography**: no overlapping text, no orphaned headings, no text under the fixed header
- [x] **Images**: logo centered and unstretched, alt text present

## Defects found and fixed during QA

1. **Horizontal page overflow on `/reference/api/` (mobile)** — wide API signature tables. Fixed with block-level scrollable tables (`max-width: 100%`).
2. **Raw `:badge[...]` text in table cells** — the `starlight-heading-badges` plugin only transforms heading directives. Fixed by using Starlight's `<Badge>` component in table cells.
3. **Heading overflow from anchor padding on mobile** — inline headings with badges + reserved anchor `padding-right` exceeded 390 px. Fixed with a ≤640 px media query resetting that padding.
4. **Draft banner on the production 404 page** — hidden on that page only via frontmatter `head` style; the draft mechanism itself is required to avoid a Starlight route-conflict warning.

Each fix was followed by a full `pnpm build` → `pnpm preview` → `pnpm qa` re-run; the final state passed all gates as the last action before packaging.
