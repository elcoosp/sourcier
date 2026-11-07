# ADR 0005: Known warnings

**Status:** Accepted
**Date:** 2026-11-07

## Current state

**The build, check and lint gates emit zero warnings.** No warning line is
allowlisted or filtered anywhere in the QA pipeline — the grep assertion in
the release procedure (`! grep -Ei '\b(warn|warning)\b'` over the captured
logs) passes against unfiltered output.

## Environment fix recorded here: `@emnapi/runtime` override

`astro check` crashed in this environment (and in a minimal, freshly
scaffolded Starlight probe project) before producing any diagnostics:

```
this.bridge.setLastError is not a function
    at NodeEnv.clearLastError (@emnapi/runtime/dist/emnapi.js:798:21)
    at Object.init (@emnapi/core/dist/emnapi-core.mjs:150:35)
    at loadNapiModuleImpl (@emnapi/core/dist/emnapi-core.mjs:7574:12)
    at @astrojs/astro2tsx/index.js:13:24
```

Root cause: `astro check` pulls `@astrojs/astro2tsx`, which loads a
napi-rs/WASM module through `@napi-rs/wasm-runtime`, whose resolved
`@emnapi/runtime@2.0.0-beta.2` is API-incompatible with the stable
`@emnapi/core@1.11.3` that calls into it. The mismatch is a dependency
resolution bug (a beta runtime satisfying a range alongside stable core),
not a project misconfiguration — reproduced identically under Node 24.22.2
and 22.23.4, and in an empty Starlight template.

**Fix:** a narrow pnpm override pins the runtime to the matching stable
line:

```json
"pnpm": {
	"overrides": {
		"@emnapi/runtime": "1.11.3"
	}
}
```

This is a dependency **fix**, not a warning suppression: with it, `astro
check` runs and reports 0 errors / 0 warnings / 0 hints, and `astro build`
behavior is unchanged. If a future `@napi-rs/wasm-runtime` / `@astrojs/
astro2tsx` release resolves compatible emnapi versions, remove the
override.

## Procedure reminder

If a future dependency introduces an unavoidable warning, record the exact
line and the reason here, and add only a narrow `grep -v` for that exact
string with a comment. Blanket suppression is forbidden.
