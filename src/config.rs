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

use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{Component, Path, PathBuf};

use anyhow::Error;
use clap::ArgMatches;
use serde::Deserialize;

use crate::error::AppError;
use crate::temp;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    Inline { inline: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Flavor {
    #[serde(rename = "os-release")]
    os_release: Option<InlineOrPath>,
    cmdline: Option<InlineOrPath>,
    #[serde(rename = "splash-image")]
    splash_image: Option<PathBuf>,
    linux: Option<PathBuf>,
    initrd: OneOrMany<PathBuf>,
    efistub: Option<PathBuf>,
    output: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Kernel {
    /// The following params are global optionals, that
    /// can be overriden with specific flavor params.
    cmdline: Option<InlineOrPath>,
    linux: Option<PathBuf>,
    #[serde(rename = "splash-image")]
    splash_image: Option<PathBuf>,
    efistub: Option<PathBuf>,

    /// Map of flavors
    pub flavors: HashMap<String, Flavor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(skip)]
    location: PathBuf,
    #[serde(flatten)]
    pub kernels: HashMap<String, Kernel>,
}

fn canonicalize(relative_to: impl AsRef<Path>, path: impl AsRef<Path>) -> PathBuf {
    if path.as_ref().is_absolute() {
        path.as_ref().into()
    } else {
        let mut result = relative_to.as_ref().to_path_buf();
        path.as_ref().components().for_each(|c| match c {
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(c) => result.push(c),
            _ => {}
        });

        result
    }
}

fn check_file(relative_to: impl AsRef<Path>, path: impl AsRef<Path>) -> Result<PathBuf, AppError> {
    let path = canonicalize(relative_to, path);
    if !path.exists() {
        Err(AppError::IoError {
            path,
            source: ErrorKind::NotFound.into(),
        })
    } else if !path.is_file() {
        Err(AppError::IoError {
            path,
            source: ErrorKind::InvalidInput.into(),
        })
    } else {
        Ok(path)
    }
}

fn check_splash(
    relative_to: impl AsRef<Path>,
    path: impl AsRef<Path>,
) -> Result<PathBuf, AppError> {
    let path = canonicalize(relative_to, path);
    let mut file = File::open(&path).map_err(|e| AppError::IoError {
        path: path.clone(),
        source: e,
    })?;

    let mut bytes = [0; 2];
    file.read_exact(&mut bytes)
        .map_err(|_| AppError::InvalidSplash)?;

    if infer::image::is_bmp(&bytes) {
        Ok(path)
    } else {
        Err(AppError::InvalidSplash)
    }
}

impl Config {
    pub fn from_config_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let config_path = matches.value_of("config").unwrap();
        let contents = std::fs::read_to_string(config_path)?;
        let mut config: Self = yaml::from_str(&contents)?;
        config.location = Path::new(config_path)
            .parent()
            .unwrap_or(&std::env::current_dir()?)
            .canonicalize()?;

        Ok(config)
    }

    pub fn from_args_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let os_release = matches
            .value_of("osrel")
            .map(|p| InlineOrPath::Path(p.into()));

        let cmdline = matches
            .value_of("cmdline")
            .map(|p| InlineOrPath::Path(p.into()));

        let splash_image = matches.value_of("splash").map(Into::into);

        let linux = matches.value_of("linux").unwrap().into();

        let initrd: Vec<PathBuf> = matches
            .values_of("initrd")
            .unwrap()
            .map(Into::into)
            .collect();

        let initrd = if initrd.len() == 1 {
            OneOrMany::One(initrd[0].clone())
        } else {
            OneOrMany::Many(initrd)
        };

        let efistub = matches.value_of("efistub").map(Into::into);

        let output = matches.value_of("output").unwrap().into();

        let flavor = Flavor {
            os_release,
            cmdline,
            splash_image,
            linux: Some(linux),
            initrd,
            efistub,
            output,
        };

        let mut flavors = HashMap::with_capacity(1);
        flavors.insert("unknown".into(), flavor);
        let kernel = Kernel {
            cmdline: None,
            linux: None,
            splash_image: None,
            efistub: None,
            flavors,
        };

        let mut kernels = HashMap::with_capacity(1);
        kernels.insert("unknown".into(), kernel);
        Ok(Self {
            location: std::env::current_dir()?,
            kernels,
        })
    }

    pub fn os_release_path(&self, kernel: &str, flavor: &str) -> Result<PathBuf, AppError> {
        let os_release = &self.kernels[kernel].flavors[flavor].os_release;

        match os_release {
            Some(InlineOrPath::Path(path)) => check_file(&self.location, path),
            Some(InlineOrPath::Inline { inline: contents }) => {
                let (path, mut temp) =
                    temp::temp_file(&format!("{}-{}-os_release", kernel, flavor))?;

                write!(temp, "{}", contents).map_err(|e| AppError::IoError {
                    path: path.clone(),
                    source: e,
                })?;

                Ok(path)
            }

            _ => check_file(&self.location, "/etc/os-release"),
        }
    }

    pub fn cmdline_path(&self, kernel: &str, flavor: &str) -> Result<Option<PathBuf>, AppError> {
        let kernel_entry = &self.kernels[kernel];
        let cmdline = kernel_entry.flavors[flavor]
            .cmdline
            .clone()
            .or_else(|| kernel_entry.cmdline.clone());

        match cmdline {
            Some(InlineOrPath::Path(path)) => Ok(Some(check_file(&self.location, path)?)),
            Some(InlineOrPath::Inline { inline: contents }) => {
                let (path, mut temp) = temp::temp_file(&format!("{}-{}-cmdline", kernel, flavor))?;

                write!(temp, "{}", contents).map_err(|e| AppError::IoError {
                    path: path.clone(),
                    source: e,
                })?;

                Ok(Some(path))
            }

            _ => Ok(None),
        }
    }

    pub fn splash_image_path(
        &self,
        kernel: &str,
        flavor: &str,
    ) -> Result<Option<PathBuf>, AppError> {
        let kernel_entry = &self.kernels[kernel];
        let splash_image = kernel_entry.flavors[flavor]
            .splash_image
            .clone()
            .or_else(|| kernel_entry.splash_image.clone());

        match splash_image {
            Some(path) => Ok(Some(check_splash(&self.location, path)?)),
            None => Ok(None),
        }
    }

    pub fn linux_path(&self, kernel: &str, flavor: &str) -> Result<PathBuf, AppError> {
        let kernel_entry = &self.kernels[kernel];
        let linux = kernel_entry.flavors[flavor]
            .linux
            .clone()
            .or_else(|| kernel_entry.linux.clone());

        match linux {
            Some(path) => check_file(&self.location, path),
            None => Err(AppError::NotProvided("linux")),
        }
    }

    pub fn initrd_path(&self, kernel: &str, flavor: &str) -> Result<PathBuf, AppError> {
        let initrd = &self.kernels[kernel].flavors[flavor].initrd;

        match initrd {
            OneOrMany::One(path) => check_file(&self.location, path),
            OneOrMany::Many(paths) => {
                let (path, mut temp) =
                    temp::temp_file(&format!("{}-{}-initrd.img", kernel, flavor))?;

                for initrd in paths {
                    let contents = std::fs::read(initrd).map_err(|e| AppError::IoError {
                        path: initrd.clone(),
                        source: e,
                    })?;

                    temp.write_all(&contents).map_err(|e| AppError::IoError {
                        path: initrd.clone(),
                        source: e,
                    })?;
                }

                Ok(path)
            }
        }
    }

    pub fn efistub_path(&self, kernel: &str, flavor: &str) -> Result<PathBuf, AppError> {
        let kernel_entry = &self.kernels[kernel];
        let efistub = kernel_entry.flavors[flavor]
            .efistub
            .clone()
            .or_else(|| kernel_entry.efistub.clone());

        match efistub {
            Some(path) => check_file(&self.location, path),
            None => check_file(
                &self.location,
                "/usr/lib/systemd/boot/efi/linuxx64.efi.stub",
            ),
        }
    }

    pub fn output_path(&self, kernel: &str, flavor: &str) -> Result<PathBuf, AppError> {
        Ok(self.kernels[kernel].flavors[flavor].output.clone())
    }
}
