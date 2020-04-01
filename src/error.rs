// Copyright (C) 2020 Kevin Dc
//
// This file is part of gen-uki.
//
// gen-uki is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// gen-uki is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with gen-uki.  If not, see <http://www.gnu.org/licenses/>.

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Splash image is not a valid bmp file")]
    InvalidSplash,

    #[error("I/O error (path: \"{}\", reason: {})", path.to_string_lossy(), source)]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Argument {0} was not provided!")]
    NotProvided(&'static str),
}
