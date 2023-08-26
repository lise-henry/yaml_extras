// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.


//! Misc yaml-related utility functions.
//!
//! # Restructure
//!
//! If you use YAML for a configuration file, you might want to allow to use
//! both things like:
//!
//! ```yaml
//! compiler:
//!     command: cargo build
//! ```
//!
//! and:
//!
//! ```yaml
//! compiler.command: cargo build
//! ```
//!
//! (Or not. I know *I* needed that. Whatever.)
//!
//! The `Restructurer` methods allow just that,
//! converting dotted keys to inner fiels:
//!
//! ```
//!         let s1 = r#"
//! compiler:
//!     command: cargo build
//! "#;
//! 
//!         let s2 = r#"
//! compiler.command: cargo build
//! "#;
//!         let v1: serde_yaml::Value = serde_yaml::from_str(s1)?;
//!         let v2 = yaml_extras::Restructurer::new()
//!             .apply_str(&s2)?;
//!         assert_eq!(v1, v2);
//! # Ok::<(), yaml_extras::Error>(())
//! ```


mod error;
mod restructure;
mod document;

pub use error::{Result, Error};
pub use restructure::Restructurer;
pub use document::Documenter;
