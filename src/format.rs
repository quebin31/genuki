// Copyright (C) 2020 Kevin Dc
//
// This file is part of genuki.
//
// genuki is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// genuki is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with genuki.  If not, see <http://www.gnu.org/licenses/>.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FormatPath(String);

impl FormatPath {
    pub fn replace(&self, kernel: &str, flavor: &str) -> PathBuf {
        let mut format = self.0.clone();

        let placeholders = ["{kernel}", "{flavor}"];

        for placeholder in &placeholders {
            if format.contains(placeholder) {
                let value = match *placeholder {
                    "{kernel}" => kernel,
                    "{flavor}" => flavor,
                    _ => unimplemented!(),
                };

                format = format.replace(placeholder, value);
            }
        }

        format.into()
    }
}

impl From<&Path> for FormatPath {
    fn from(path: &Path) -> Self {
        Self(path.to_string_lossy().to_string())
    }
}

impl From<String> for FormatPath {
    fn from(string: String) -> Self {
        Self(string)
    }
}
