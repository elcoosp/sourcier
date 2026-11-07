# ADR 0001: Pin TypeScript to 6.x for `astro check`

**Status:** Accepted
**Date:** 2026-11-07

## Context

`astro check` (via `@astrojs/check`) does not support TypeScript 7. Running
`astro check` with TypeScript 7.0.2 installed fails with:

> astro check does not currently support TypeScript 7.0. To continue using
> astro check, install TypeScript 6 instead.
>
> astro check will be deprecated in a future Astro release. Experimental
> support for type-checking Astro files with TypeScript 7.1+ is available
> through @astrojs/ts-content-mapper. See its README for setup instructions:
> https://github.com/withastro/astro/tree/main/packages/language-tools/ts-content-mapper#usage

(Quoted verbatim from `astro check` output on Astro 7.4.7, November 2026.)

## Decision

Pin `typescript` to the latest 6.x release (**6.0.3**, verified via
`pnpm view typescript versions --json`) in `devDependencies`. Do not adopt
the experimental `@astrojs/ts-content-mapper` path.

## Consequences

- `astro check` reports **0 errors, 0 warnings, 0 hints** on this project.
- When Astro ships stable TypeScript 7 support, upgrade in one commit and
  re-run the full gate.
