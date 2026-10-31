# sourcier-core

Core implementation of [sourcier](https://github.com/elcoosp/sourcier): a
library for compact source file mapping and position tracking.

This crate stores the actual engine (file registry, packed positions, line
offset indexes and runtime feedback). Most users should depend on the
`sourcier` facade crate instead, which re-exports this entire API:

```toml
[dependencies]
sourcier = "1.0"
```

See the [workspace README](https://github.com/elcoosp/sourcier) and the
[docs.rs documentation](https://docs.rs/sourcier) for the full guide,
feature flags and performance notes.

Licensed under the [MIT license](../LICENSE-MIT).
