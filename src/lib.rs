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
//! The functions `restructure_map` and `restructure_from_str` allow just that,
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
//!         let v1: serde_yaml::Value = serde_yaml::from_str(s1).unwrap();
//!         let v2 = yaml_extras::restructure_from_str(&s2, true).unwrap();
//!         assert_eq!(v1, v2);
//! ```


mod error;
mod restructure;
mod document;

pub use restructure::restructure_from_str;
pub use restructure::restructure_map;
pub use document::document;
