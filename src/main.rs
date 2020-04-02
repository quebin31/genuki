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

mod app;
mod config;
mod error;
mod logger;
mod temp;

use anyhow::Error;
use clap::{App as ClapApp, Arg};

use crate::app::App;
use crate::logger::init_logger;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

fn run(app: ClapApp) -> Result<(), Error> {
    let matches = app.get_matches();
    let verbose = matches.occurrences_of("verbose");

    init_logger(verbose).unwrap();

    log::debug!("Parsing arguments (and maybe config)");
    let app = App::from_matches(matches)?;
    log::debug!("Parsed app: {:#?}", &app);
    app.run()
}

fn main() {
    let app = ClapApp::new(NAME)
        .version(VERSION)
        .author(AUTHORS)
        .about(ABOUT)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Set verbosity level (multiple)"),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value("/etc/genuki/config.yaml")
                .help("Set custom config file"),
        )
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Generate UKIs for all the entries"),
        )
        .arg(
            Arg::with_name("entries")
                .value_name("REGEX")
                .min_values(1)
                .index(1)
                .required_unless("all")
                .help("Generate UKIs for the specified regexes"),
        );

    if let Err(e) = run(app) {
        log::error!("{}", e);
        std::process::exit(1);
    }
}
