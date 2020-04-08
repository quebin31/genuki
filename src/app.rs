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

use anyhow::Error;
use clap::ArgMatches;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct App {
    config: Config,
    to_build: Vec<(String, String)>,
}

impl App {
    pub fn from_matches(matches: ArgMatches) -> Result<Self, Error> {
        let config = Config::from_matches(&matches)?;

        let mut all_entries = Vec::new();
        for (name, kernel) in &config.kernels {
            for flavor in kernel.flavors.keys() {
                all_entries.push((name.to_string(), flavor.to_string()));
            }
        }

        let to_build = if matches.is_present("all") {
            all_entries
        } else {
            let regexes: Vec<_> = matches.values_of("entries").unwrap().collect();
            let regexes = regex::RegexSet::new(&regexes)?;

            let to_build: Vec<_> = all_entries
                .iter()
                .filter_map(|(kernel, flavor)| {
                    let human_name = format!("{}.{}", kernel, flavor);
                    if regexes.is_match(&human_name) {
                        Some((kernel.to_string(), flavor.to_string()))
                    } else {
                        None
                    }
                })
                .collect();

            to_build
        };

        Ok(Self { config, to_build })
    }

    pub fn run(&self) -> Result<(), Error> {
        for (kernel, flavor) in &self.to_build {
            if !self.config.is_enabled(kernel, flavor) {
                continue;
            }

            self.generate_for(&kernel, &flavor)?;
        }

        Ok(())
    }

    fn generate_for(&self, kernel: &str, flavor: &str) -> Result<(), Error> {
        let objcopy_path = which::which_in("objcopy", Some("/usr/bin/"), std::env::current_dir()?)?;

        let mut cmd_args = vec![
            // os-release section
            "--add-section".into(),
            format!(
                ".osrel=\"{}\"",
                self.config
                    .os_release_path(kernel, flavor)?
                    .to_string_lossy()
            ),
            "--change-section-vma".into(),
            ".osrel=0x20000".into(),
        ];

        if let Some(cmdline) = self.config.cmdline_path(kernel, flavor)? {
            cmd_args.push("--add-section".into());
            cmd_args.push(format!(".cmdline=\"{}\"", cmdline.to_string_lossy()));
            cmd_args.push("--change-section-vma".into());
            cmd_args.push(".cmdline=0x30000".into());
        }

        if let Some(splash_image) = self.config.splash_image_path(kernel, flavor)? {
            cmd_args.push("--add-section".into());
            cmd_args.push(format!(".splash=\"{}\"", splash_image.to_string_lossy()));
            cmd_args.push("--change-section-vma".into());
            cmd_args.push(".splash=0x40000".into());
        }

        cmd_args.extend_from_slice(&[
            // Linux image section
            "--add-section".into(),
            format!(
                ".linux=\"{}\"",
                self.config.linux_path(kernel, flavor)?.to_string_lossy()
            ),
            "--change-section-vma".into(),
            ".linux=0x2000000".into(),
            // Initrd image section
            "--add-section".into(),
            format!(
                ".initrd=\"{}\"",
                self.config.initrd_path(kernel, flavor)?.to_string_lossy()
            ),
            "--change-section-vma".into(),
            ".initrd=0x3000000".into(),
            // EFI stub file
            format!(
                "\"{}\"",
                self.config.efistub_path(kernel, flavor)?.to_string_lossy()
            ),
            // Output file
            format!(
                "\"{}\"",
                self.config.output_path(kernel, flavor)?.to_string_lossy()
            ),
        ]);

        log::info!("Generating unified kernel image for {}.{}", kernel, flavor);
        let parent: PathBuf = self
            .config
            .output_path(kernel, flavor)?
            .parent()
            .map_or(std::env::current_dir()?, |p| p.into());

        log::debug!("Arguments for objcopy: {:#?}", &cmd_args);

        maybe_create_dir(parent)?;
        shells::wrap_sh!("{} {}", objcopy_path.to_string_lossy(), cmd_args.join(" "))?;
        log::info!("Successfully generated!");
        Ok(())
    }
}

fn maybe_create_dir(path: impl AsRef<Path>) -> std::io::Result<()> {
    match std::fs::create_dir_all(path) {
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
        Ok(_) => Ok(()),
    }
}
