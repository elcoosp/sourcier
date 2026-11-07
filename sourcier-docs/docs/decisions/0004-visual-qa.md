# ADR 0004: Visual QA toolchain

**Status:** Accepted
**Date:** 2026-11-07

## Context

A green build does not prove the site renders correctly. The QA phase
requires every built route to be rendered and inspected in both viewports
(desktop 1440×900, mobile 390×844) and both themes, with automated probes
and a manual eyeball pass.

## Decision

- **Playwright** drives headless Chromium (`pnpm exec playwright install
  chromium`; the headless shell is sufficient — no system deps were
  installed, `--with-deps` is unnecessary in this environment).
- `scripts/qa-screenshots.mjs` enumerates routes **from `dist/`** (walking
  for `index.html`), so no page can be skipped by a hand-maintained list. It
  captures every route × 2 viewports × 2 themes as full-page screenshots
  into `.qa/screenshots/` and fails on: console errors, uncaught page
  errors, HTTP ≥ 400, horizontal overflow, missing/duplicate `<h1>`, main
  text < 200 chars, theme-application failure, or broken `<img>`.
- `scripts/manual-checks.mjs` covers the two interactive checks the probes
  cannot: the mobile hamburger (`button.sl-menu-button` →
  `starlight__sidebar` popover) and the Pagefind search modal (Ctrl+K,
  results for a domain term).
- The eyeball pass reads the actual screenshots; its outcome is recorded in
  `docs/qa-report.md`.
- `.qa/` is gitignored and excluded from the deliverable archive.

## Not adopted

- `@axe-core/playwright` extra hardening was considered but not added in
  this pass; the probes cover contrast-critical structure (theme
  application, headings, overflow, images). Revisit as a follow-up.

## Consequences

- Any config, CSS, logo, theme or content change invalidates the QA run;
  re-run `pnpm build` → `pnpm preview` → `pnpm qa` afterwards.
