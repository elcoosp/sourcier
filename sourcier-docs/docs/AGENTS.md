# AGENTS.md — Sourcier documentation site

Guide for AI agents and humans working on this Starlight documentation
codebase for [elcoosp/sourcier](https://github.com/elcoosp/sourcier).

## What this is

A standalone Astro **7.4.7** + Starlight **0.42.10** documentation site
covering the `sourcier` Rust crate (v2.0.0): compact source file mapping and
position tracking. Content follows the Diátaxis framework — one type per
page, declared in the `type` frontmatter field (validated by the extended
schema in `src/content.config.ts`).

| Path | Purpose |
| --- | --- |
| `src/content/docs/` | All pages (MDX). `index.mdx` = splash landing; `tutorials/`, `guides/`, `reference/`, `explanation/` = Diátaxis sections; `404.mdx` = custom 404 (draft, see decisions/0006) |
| `src/content/i18n/root.json` | Root-locale UI translation file (kept so the i18n collection is non-empty — prevents a build warning) |
| `src/content.config.ts` | Collections: `docs` (with Diátaxis `type` extension) and `i18n` |
| `astro.config.mjs` | Starlight config: sidebar, plugins, branding |
| `src/styles/custom.css` | Brand accent overrides (light + dark via `[data-theme]`) and table/heading fixes |
| `src/assets/logo.svg` | SVGO-optimized SVG traced from the repo's PNG logo (viewBox preserved) |
| `public/favicon.svg` | Small-size favicon traced for 16–32 px rendering |
| `scripts/qa-screenshots.mjs` | Visual QA: every dist route × 2 viewports × 2 themes → `.qa/screenshots/` |
| `scripts/manual-checks.mjs` | Interactive checks: mobile hamburger drawer + Pagefind search modal |
| `docs/decisions/` | ADRs: TypeScript pin, logo tracing, Shiki, QA toolchain, warnings, site/llms config, omitted plugins |
| `docs/qa-report.md` | Per-route eyeball results of the final QA pass |

## Commands (pnpm only)

```bash
pnpm install        # dependencies
pnpm dev            # dev server at localhost:4321
pnpm build          # production build to dist/ (must exit 0 with ZERO warnings)
pnpm preview        # serve dist/
pnpm check          # astro check — must report 0 errors, 0 warnings, 0 hints
pnpm lint           # markdownlint-cli2 over src/content/docs — must be clean
pnpm qa             # screenshot suite against a running preview (release blocker)
```

The full release gate: `pnpm build`, `pnpm check`, `pnpm lint` clean with
zero warnings, then `pnpm preview --port 4321 &` and `pnpm qa` exiting 0.
Re-run the gate after ANY change to config, CSS, logo, theme or content.
Do not filter warnings; if a third-party warning is truly unfixable, record
it in `docs/decisions/0005-known-warnings.md` with a narrow grep exclusion.

## Content rules

- Frontmatter on every page: `title`, `description`, `type`
  (`tutorial | how-to | reference | explanation`).
- Second person, present tense, active voice, imperative instructions.
  Never "simply", "just", "easy", "obviously".
- Every code block declares a language (`rust`, `toml`, `bash`, `ansi`).
- Feature-gated APIs are marked with `:badge[view]` / `:badge[rt-feedback]`
  in **headings** (the heading-badges plugin only processes headings) and
  with `<Badge text="..." size="small" />` (imported from
  `@astrojs/starlight/components`) everywhere else — e.g. table cells.
- Internal links are root-relative without `base` and must resolve —
  `starlight-links-validator` fails the build otherwise.
- All facts must come from the crate source, README, CHANGELOG.md or
  PERFORMANCE.md in the sourcier repository. Never invent API surface.

## Environment notes

- TypeScript is pinned to 6.x (7.x breaks `astro check`) — see
  decisions/0001.
- `@emnapi/runtime` is pinned to `1.11.3` via a pnpm override (beta runtime
  breaks `astro check`'s astro2tsx WASM bridge) — see decisions/0005.
- `site` is set to a GitHub Pages convention URL and `base` is unset — see
  decisions/0006.
