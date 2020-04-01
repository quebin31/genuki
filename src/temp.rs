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

use std::fs::File;
use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn temp_file(name: &str) -> Result<(PathBuf, File), AppError> {
    let path = Path::new("/tmp").join(name);

    log::debug!("Creating temp file: /tmp/{}", name);
    if path.exists() {
        let remove = if path.is_dir() {
            std::fs::remove_dir
        } else {
            std::fs::remove_file
        };

        remove(&path).map_err(|e| AppError::IoError {
            path: path.clone(),
            source: e,
        })?;
    }

    let file = File::create(&path).map_err(|e| AppError::IoError {
        path: path.clone(),
        source: e,
    })?;

    Ok((path, file))
}
