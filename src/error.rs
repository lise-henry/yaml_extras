// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("impossible to mege YAML values: {0}")]
    Merge(String),
    #[error("impossible to restructure YAML map: {0}")]
    Restructure(String),
    #[error("YAML error")]
    Yaml(#[from] serde_yaml::Error)
}

pub type Result<T> = std::result::Result<T, Error>;
