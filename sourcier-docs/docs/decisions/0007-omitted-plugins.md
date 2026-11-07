# ADR 0007: Omitted plugins

**Status:** Accepted
**Date:** 2026-11-07

The "bells and whistles" plugin set was evaluated against Starlight 0.42.10
/ Astro 7.4.7. Installed plugins are recorded in `astro.config.mjs`; this
document records every omission and the reason.

## Installed

| Plugin | Why |
| --- | --- |
| `starlight-links-validator` 0.27.0 | Fails the build on broken internal links — a release blocker |
| `starlight-image-zoom` 0.16.2 | Click-to-zoom for documentation images, lightweight, accessible |
| `starlight-heading-badges` 0.8.0 | Feature-gated API headings (`:badge[view]`, `:badge[rt-feedback]`) in the reference section |
| `starlight-llms-txt` 0.12.0 | Generates `llms.txt` / `llms-small.txt` / `llms-full.txt` for AI-agent discoverability |

## Omitted

| Plugin | Reason |
| --- | --- |
| `starlight-typedoc` | Generates API reference from **TypeScript** sources. Sourcier is a Rust crate; its generated API reference lives on docs.rs, and the hand-written reference section (`/reference/api/`) documents the public surface here. Nothing for typedoc to consume. |
| `starlight-openapi` | No OpenAPI specification exists in the repository. |
| `starlight-blog` | The project's changelog is rendered as a reference page (`/reference/changelog/`) sourced from the git-cliff-generated CHANGELOG.md; a blog section adds routes and UI surface no content currently fills. Revisit if the project starts publishing release essays. |
| `starlight-versions` | Single current major version (2.0); versioned docs would be empty scaffolding, and the plugin is marked early-development by its own documentation. Revisit at 3.0. |

## MDX content components used

`Card`, `CardGrid`, `LinkCard`, `Tabs`/`TabItem`, `Steps`, `Aside`,
`Badge`, plus `FileTree`-free structure (the project is a library, not a
scaffold — no file tree is documented).
