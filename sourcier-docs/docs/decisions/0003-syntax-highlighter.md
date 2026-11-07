# ADR 0003: Syntax highlighting — Shiki via Expressive Code (Starlight default)

**Status:** Accepted
**Date:** 2026-11-07

## Context

Starlight renders code blocks through Expressive Code, whose supported
highlighter is Shiki (TextMate grammars, 180+ languages, build-time
execution, zero client-side JavaScript, dual light/dark themes). A
lightweight alternative (Rangi, ~13 kB, ~46 languages, regex tokenizer) was
evaluated.

## Decision

Ship with Shiki — the Starlight default. Do not add Rangi or any custom
highlighter plugin.

Rationale, following the comparison table in the Rangi documentation itself:

| | Rangi | Shiki |
| --- | --- | --- |
| Bundle size | Tiny (~13 kB all-in) | Larger (TextMate grammars) |
| Lexing | Regex tokenizer | TextMate grammars |
| Theming | Built-in themes / pairs | Dual themes + CSS vars |
| Transformers / Twoslash | No | Yes |
| Language coverage | ~46 languages | 180+ |
| Best for | Chat UIs, SSG, small apps | Docs sites, rich code samples |

For a documentation site, Shiki is the designated highlighter. Rangi is not
integrated with Expressive Code, so using it would require maintaining a
custom highlighter plugin with no upstream support while losing
transformers, Twoslash and the broad language set (this site needs `rust`,
`toml`, `bash`, `ansi`, `mdx`, `js`).

## Configuration

No `expressiveCode` overrides are set: Starlight's default Shiki setup
(dual GitHub themes, per-language loading on demand, copy button, line
highlighting support) is already correct for this site. The default is kept
intentionally to minimize config surface.

## Authoring rules enforced

- Every fenced code block declares a language.
- Terminal output uses the `ansi` tag.
- Snippets are minimal and drawn from the crate's own doctests and README.
