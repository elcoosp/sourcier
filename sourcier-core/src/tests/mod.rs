//! Integration-style tests, compiled as part of the library's own test
//! binary.
//!
//! Keeping the whole suite in one test binary is deliberate: every generic
//! instantiation is then covered exactly once, which keeps coverage
//! accounting free of phantom misses from duplicate copies across separate
//! test crates.

pub mod builder;
pub mod feedback;
pub mod lifecycle;
#[cfg(feature = "view")]
pub mod navigation;
#[cfg(all(feature = "view", feature = "parallel"))]
pub mod parallel;
pub mod positions;
pub mod property;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "serde")]
pub mod snapshots;
#[cfg(feature = "view")]
pub mod view;
