# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [2.0.0] - 2026-11-14

### Bug Fixes
- gate RuntimeFeedback import behind rt-feedback by @elcoosp ([2abeb15](https://github.com/elcoosp/sourcier/commit/2abeb1571f2b7777f6cca8b3a4131e098be20943))
- use correct RuntimeFeedback path by @elcoosp ([a605306](https://github.com/elcoosp/sourcier/commit/a6053062665fb340c09de3ad68ae6b95ae5b6372))
- ungate SourceFilePosition import by @elcoosp ([9b30db7](https://github.com/elcoosp/sourcier/commit/9b30db72c4ea303121f1da8bd4ec490128594a7d))
- remove duplicate SourceFilePosition import by @elcoosp ([bf47576](https://github.com/elcoosp/sourcier/commit/bf475762cf6914262fe003dfe55a1aa000e822d6))
- correct module paths for SourceFilePosition and RuntimeFeedback by @elcoosp ([5fbd919](https://github.com/elcoosp/sourcier/commit/5fbd919e266a605a60f6b5faa345d16de0d178ed))
- move import below doc comments (E0753) by @elcoosp ([7941eac](https://github.com/elcoosp/sourcier/commit/7941eac641cfcbec32074eb2badbe729bf61c1c7))
- add missing SourceFilePosition import by @elcoosp ([2a8bfe5](https://github.com/elcoosp/sourcier/commit/2a8bfe5ccc6986a884a39b740370e8737d93a065))
- gate unix-only tests by @elcoosp ([6346b1a](https://github.com/elcoosp/sourcier/commit/6346b1ae42af180f3276e292257b41a7f3b68098))
- resolve wildcard dep and gate unix tests by @elcoosp ([9228003](https://github.com/elcoosp/sourcier/commit/9228003d8676426a63390e46388434bafa4d939a))
- rewrite corrupted sourcier-facade/Cargo.toml ([b325c7f](https://github.com/elcoosp/sourcier/commit/b325c7f173db803960b638c4b8f65b5961f85b67))
- replace external paths with git URLs + inline workspace deps ([7f3b782](https://github.com/elcoosp/sourcier/commit/7f3b7828098d8058a022a440dd37aca64c93a4e6))
- fix remaining warns by @elcoosp ([140cc27](https://github.com/elcoosp/sourcier/commit/140cc27d631d7efa303894280c3dc42ecf40dea7))
- test issues by @elcoosp ([229d8c8](https://github.com/elcoosp/sourcier/commit/229d8c8c8289ba25cdf410f3af19258fb6693119))
- dep:memchr for view feat by @elcoosp ([4970dd5](https://github.com/elcoosp/sourcier/commit/4970dd567295c30b5539cd9444dd40429f527a3f))
- missing clo git track & mv SourceFilesMap defaults to self + by @elcoosp ([3706e1c](https://github.com/elcoosp/sourcier/commit/3706e1c29e139460c6e909d9d1d1f9a16608e488))
- mv clo mod & use memchr by @elcoosp in [#3](https://github.com/elcoosp/sourcier/pull/3) ([4929fd0](https://github.com/elcoosp/sourcier/commit/4929fd0a89d84809ff6c16c2893142a8a5e7e0e8))

### CI
- missing fields for release by @elcoosp ([7e840cb](https://github.com/elcoosp/sourcier/commit/7e840cb08b049bf29daa9fcf1cb8e2c89e8d7241))

### Documentation
- add Vercel documentation link to Cargo.toml by @elcoosp ([d5ab8a5](https://github.com/elcoosp/sourcier/commit/d5ab8a5b5d586b7bc474a83bce3169f21a73c254))
- enforce SKILL.md rules (jsDelivr logos, absolute links, remove license sections) by @elcoosp ([b235e8c](https://github.com/elcoosp/sourcier/commit/b235e8c8f5855772b613252c1fc90ba57d29eff1))

### Features
- wire the builder module and re-export the new types by @elcoosp ([d6a176b](https://github.com/elcoosp/sourcier/commit/d6a176bab5134940fe82e704e163ec30cf80bcc4))
- add the type-state SourceFilesBuilder by @elcoosp ([e4d5b97](https://github.com/elcoosp/sourcier/commit/e4d5b97b93d2cc350576b9f4d5cfc42e341d908b))
- add batch/path registration, offset bridge and PositionDisplay by @elcoosp ([d35b11c](https://github.com/elcoosp/sourcier/commit/d35b11c8320d2790c68a116f3751bef4e4d126d8))
- const-ify constructors and add span conversions by @elcoosp ([a04172b](https://github.com/elcoosp/sourcier/commit/a04172bd0dba25d69030a13e88af7994f1db5fb1))
- add MapError::PathNotUtf8 for non-UTF-8 path registration by @elcoosp ([841078d](https://github.com/elcoosp/sourcier/commit/841078df04666992c169b19bc32b1aa19652ee2d))
- build verified open-addressed index and inline path storage by @elcoosp ([b548149](https://github.com/elcoosp/sourcier/commit/b548149affe1d1d2ae3ca3f86a9e0764dc94ef07))
- expose line starts accessor and optimize vector initialization by @elcoosp ([a5b78b1](https://github.com/elcoosp/sourcier/commit/a5b78b10ca9a855ca345d5141a77b7b99e6d2ace))
- update metadata, dependencies, and benchmark setup by @elcoosp ([cf99770](https://github.com/elcoosp/sourcier/commit/cf997701327f129c49252c366de1c4e8ae44a91b))
- add serde optional by @elcoosp in [#7](https://github.com/elcoosp/sourcier/pull/7) ([1e8f70a](https://github.com/elcoosp/sourcier/commit/1e8f70a0a6757b8683df0878578a42c4b7b59616))
- view poc by @elcoosp in [#3](https://github.com/elcoosp/sourcier/pull/3) ([8a8f6f5](https://github.com/elcoosp/sourcier/commit/8a8f6f57514235b812a939b9689b6f3bbe0f8d39))
- poc RuntimeFeedback (with_feedback fn) by @elcoosp in [#2](https://github.com/elcoosp/sourcier/pull/2) ([2317299](https://github.com/elcoosp/sourcier/commit/23172999bd61fd3959ff3db061df0c42e969c8f5))
- include content in SourceFilesMap create FileEntry underneath, sort using content length after path, preallocate & add `get_content` by @elcoosp in [#1](https://github.com/elcoosp/sourcier/pull/1) ([c7b828a](https://github.com/elcoosp/sourcier/commit/c7b828a4d9528bed27e1ae77d127868db6215f63))
- add fid, sfm and sfp mods to manage source file position in a compact u64 repr, and mapping path to id in O(1), utils in sfp by @elcoosp ([9e91a7c](https://github.com/elcoosp/sourcier/commit/9e91a7cbe753dcf93f184385a28b297ceacb6e6a))

### Miscellaneous
- inject missing metadata and point to root README by @elcoosp ([c98ff5e](https://github.com/elcoosp/sourcier/commit/c98ff5eab93afec03401cf137045dc7eb5064b43))
- inject precisely curated keywords for crates.io by @elcoosp ([34520ee](https://github.com/elcoosp/sourcier/commit/34520ee8dfd061f8c051874b9a63f1b4aef3e8e4))
- configure optional rayon dependency requirement by @elcoosp ([f747414](https://github.com/elcoosp/sourcier/commit/f747414c8d3a8b8b692b9f21f4917088fc3ebd63))
- license deny cliff changelog logo and ci workflows by @elcoosp ([9e4f60f](https://github.com/elcoosp/sourcier/commit/9e4f60f535b103c7d011e0bdf74c06b66a27ae88))
- remove obsolete core changelog by @elcoosp ([8878707](https://github.com/elcoosp/sourcier/commit/8878707dc4b4d0edcbb0a42a306b6741cc840695))
- release v0.1.0 by @github-actions[bot] in [#32](https://github.com/elcoosp/sourcier/pull/32) ([81b479a](https://github.com/elcoosp/sourcier/commit/81b479ae92dcd31325340d0aa3b93066a195330f))
- clippy by @elcoosp in [#8](https://github.com/elcoosp/sourcier/pull/8) ([f73e08a](https://github.com/elcoosp/sourcier/commit/f73e08adba64a46d3f6179051171fe71f04249bf))
- clippy by @elcoosp ([613e421](https://github.com/elcoosp/sourcier/commit/613e421002212292788aee680ec7b9561e0919c3))

### Performance
- add builder and offset-bridge benchmark tracks by @elcoosp ([429f8d2](https://github.com/elcoosp/sourcier/commit/429f8d2674fe95274ca30b661fb8f26ac9cb8e4a))
- add misses testing tracks and line-col navigation benchmarks by @elcoosp ([bedf4db](https://github.com/elcoosp/sourcier/commit/bedf4dbeeb4139f5027ef3d31e4dc9b4e97a9af6))

### Refactor
- enrich top level documentation mapping memory footprint metrics by @elcoosp ([a33a1f6](https://github.com/elcoosp/sourcier/commit/a33a1f6c0c5b4dfd089aff38ee6e343332261252))
- remove obsolete sfp module by @elcoosp ([76abaa3](https://github.com/elcoosp/sourcier/commit/76abaa3eaa23558449a05c1cb7f13d1a50c460bd))
- remove obsolete sfm module by @elcoosp ([081c9f5](https://github.com/elcoosp/sourcier/commit/081c9f552b3cad582ab864368ab84ef63eb8e960))
- restructure modules and improve crate-level documentation by @elcoosp ([ee7532d](https://github.com/elcoosp/sourcier/commit/ee7532d73494aaadefacaa551f29022287fd0bbd))
- remove obsolete fid module by @elcoosp ([90006f0](https://github.com/elcoosp/sourcier/commit/90006f06020be82d916e2fb01d33c9fffcc3bdb7))
- remove obsolete clo module by @elcoosp ([9c21a35](https://github.com/elcoosp/sourcier/commit/9c21a35de23a6745c31b8f4237952046d43669be))

### Testing
- register the builder and navigation test modules by @elcoosp ([bf09db8](https://github.com/elcoosp/sourcier/commit/bf09db8bc7c2c06082cfb145c75b711118b56c76))
- cover span conversions and const constructors by @elcoosp ([c36c9f8](https://github.com/elcoosp/sourcier/commit/c36c9f8ed343bb17b8e141449a0b177fb37d21d8))
- cover add_path and by-reference iteration on the raw map by @elcoosp ([82a7c30](https://github.com/elcoosp/sourcier/commit/82a7c305ac39fc5b0ee655fb38f65c73fe0e4843))
- cover the offset bridge and diagnostic rendering by @elcoosp ([4170c59](https://github.com/elcoosp/sourcier/commit/4170c59faa5bd256cdb3826c9824ef83c29b74ba))
- cover the type-state builder and atomic batches by @elcoosp ([3793c42](https://github.com/elcoosp/sourcier/commit/3793c42fa9c7347011449338121f21196fd287e0))
- implement test cases checking raw offset ranges accessors by @elcoosp ([36d5fbc](https://github.com/elcoosp/sourcier/commit/36d5fbc2a985b85135a7947da5e95fd85161d6a7))
- update snapshot file assertions tracking lookup tables removal by @elcoosp ([ae854ab](https://github.com/elcoosp/sourcier/commit/ae854ab5b9a3ee9962e9e9c34d5ba0eecb7e8962))
- update snapshot file assertions tracking lookup tables removal by @elcoosp ([12bd9dc](https://github.com/elcoosp/sourcier/commit/12bd9dc6a79c2d8d33b104a7f8ef97a08d3c4df0))
- test wire payload backward compatibility and tolerant reads by @elcoosp ([dc2b621](https://github.com/elcoosp/sourcier/commit/dc2b621f3ff5123f4aea2e427ebd98c34281fdcc))
- property-test first-registration duplicate tiebreaks by @elcoosp ([2e67840](https://github.com/elcoosp/sourcier/commit/2e67840e9d2c26a7ca65bb0434d20b0d539567f1))
- verify parallel line-offset index construction correctness by @elcoosp ([20e7d60](https://github.com/elcoosp/sourcier/commit/20e7d608653af3dd9c6be5e427c60d65b79d380c))
- map sub-component entry configurations for parallel suites by @elcoosp ([b943532](https://github.com/elcoosp/sourcier/commit/b9435329db640e095834b8f07c8c4eeaf449aa85))
- remove obsolete snapshot file by @elcoosp ([e8fe608](https://github.com/elcoosp/sourcier/commit/e8fe6083ce58126b874e2282e4eb28b87daf083d))
- remove obsolete snapshot file by @elcoosp ([8ea54c0](https://github.com/elcoosp/sourcier/commit/8ea54c05765e10b145362b71b797237a7c53829e))
- remove obsolete snapshot file by @elcoosp ([b2195e8](https://github.com/elcoosp/sourcier/commit/b2195e847f43f5524559c36550e98ebd4aa813bf))
- remove obsolete snapshot file by @elcoosp ([543d8ec](https://github.com/elcoosp/sourcier/commit/543d8eca2ba165c894d24b65d3e7d7fbf73e9c00))
- remove obsolete snapshot file by @elcoosp ([60f6551](https://github.com/elcoosp/sourcier/commit/60f65511d36eeaf8bba916e2c141a435520135f1))
- remove obsolete snapshot file by @elcoosp ([f90ab94](https://github.com/elcoosp/sourcier/commit/f90ab943345432f3edb020ae3d449ec5b4fbbbfa))
- remove obsolete snapshot file by @elcoosp ([35edaf7](https://github.com/elcoosp/sourcier/commit/35edaf7b494bb15377a7b83bd7133d6cfc52849d))
- remove obsolete snapshot file by @elcoosp ([dd82b7c](https://github.com/elcoosp/sourcier/commit/dd82b7c4abf42b1f7d020ff476fdc83d24239a3a))
- remove obsolete snapshot file by @elcoosp ([23c3d56](https://github.com/elcoosp/sourcier/commit/23c3d56708ba22e898b7980eb85956f5583e3996))
- remove obsolete snapshot file by @elcoosp ([e7d5fcb](https://github.com/elcoosp/sourcier/commit/e7d5fcb8eb2bee3c8961eb0ad2f404646c4facf4))
- remove obsolete snapshot file by @elcoosp ([6a1910d](https://github.com/elcoosp/sourcier/commit/6a1910dffa2620cd63d7612593717f5d3ecc24e3))
- remove obsolete snapshot file by @elcoosp ([752035b](https://github.com/elcoosp/sourcier/commit/752035bcbac0b60cea0de9f86d018a9f8d765109))
- remove obsolete monolithic tests file by @elcoosp ([435be6b](https://github.com/elcoosp/sourcier/commit/435be6b1de28da03ba78fb73243c3327c3e2900f))
- use exhaustive_test_suite macro by @elcoosp in [#8](https://github.com/elcoosp/sourcier/pull/8) ([08a2d77](https://github.com/elcoosp/sourcier/commit/08a2d77d7a95fbd0b552ac0deddba7375cf0c036))
- more cases by @elcoosp in [#8](https://github.com/elcoosp/sourcier/pull/8) ([1232a4a](https://github.com/elcoosp/sourcier/commit/1232a4a73d1b9f7fbfd6a476bac8bb515135ec91))
- use yaml & redactions by @elcoosp in [#8](https://github.com/elcoosp/sourcier/pull/8) ([4f2b9c2](https://github.com/elcoosp/sourcier/commit/4f2b9c209467f3311c9a5142c2373a7f2033097f))
- add exhaustive_test_suite macro to automate snapshots by @elcoosp in [#8](https://github.com/elcoosp/sourcier/pull/8) ([902b645](https://github.com/elcoosp/sourcier/commit/902b645e8a4bc4148f5413a8e87dac53f589c3cf))
- add test_suite macro with basic multiline view by @elcoosp in [#8](https://github.com/elcoosp/sourcier/pull/8) ([4a7d3d5](https://github.com/elcoosp/sourcier/commit/4a7d3d5d268b4ada471e464a630bcc60fb547b4f))
- add test_utils by @elcoosp in [#8](https://github.com/elcoosp/sourcier/pull/8) ([49f6c1f](https://github.com/elcoosp/sourcier/commit/49f6c1f78a4e9d27ad9c8fab1246bb113383d910))
- add example integration by @elcoosp ([01a84d1](https://github.com/elcoosp/sourcier/commit/01a84d1cc7fdb503a63832b83dcd4ef3b723034f))

### style
- cargo fmt by @elcoosp ([7f36e50](https://github.com/elcoosp/sourcier/commit/7f36e50c3558fd992ad305c927e99c4802849aed))
**Full Changelog**: https://github.com/elcoosp/sourcier/releases/tag/v2.0.0
