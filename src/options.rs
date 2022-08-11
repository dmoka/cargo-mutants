// Copyright 2021, 2022 Martin Pool

//! Global in-process options for experimenting on mutants.
//!
//! The [Options] structure is built from command-line options and then widely passed around.

use std::convert::TryFrom;
use std::time::Duration;

use camino::Utf8PathBuf;
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::*;

/// Options for running experiments.
#[derive(Default, Debug, Clone)]
pub struct Options {
    /// Don't run the tests, just see if each mutant builds.
    pub check_only: bool,

    test_timeout: Duration,

    pub print_caught: bool,
    pub print_unviable: bool,

    pub show_times: bool,

    /// Show logs even from mutants that were caught, or source/unmutated builds.
    pub show_all_logs: bool,

    /// Test mutants in random order.
    ///
    /// This is now the default, so that repeated partial runs are more likely to find
    /// interesting results.
    pub shuffle: bool,

    /// Additional arguments for every cargo invocation.
    pub additional_cargo_args: Vec<String>,

    /// Additional arguments to `cargo test`.
    pub additional_cargo_test_args: Vec<String>,

    /// Copy the `/target/` directory from the source tree.
    pub copy_target: bool,

    /// Build the source directory before copying it.
    pub build_source: bool,

    /// Files to examine.
    pub examine_globset: Option<GlobSet>,

    /// Files to exclude
    pub exclude_globset: Option<GlobSet>,

    /// Create `mutants.out` within this directory (by default, the source directory).
    pub output_in_dir: Option<Utf8PathBuf>,
}

impl Options {
    /// Return the maximum run time for `cargo test` commands.
    ///
    /// Build and check are not affected.
    pub fn test_timeout(&self) -> Duration {
        self.test_timeout
    }

    pub fn has_test_timeout(&self) -> bool {
        self.test_timeout < Duration::MAX
    }

    pub fn set_test_timeout(&mut self, test_timeout: Duration) {
        self.test_timeout = test_timeout;
    }
}

impl TryFrom<&Args> for Options {
    type Error = anyhow::Error;

    fn try_from(args: &Args) -> std::result::Result<Options, anyhow::Error> {
        let examine_globset = build_glob_set(&args.file)?;
        let exclude_globset = build_glob_set(&args.exclude)?;

        Ok(Options {
            additional_cargo_args: args.cargo_arg.clone(),
            additional_cargo_test_args: args.cargo_test_args.clone(),
            build_source: !args.no_copy_target,
            check_only: args.check,
            copy_target: !args.no_copy_target,
            examine_globset,
            exclude_globset,
            output_in_dir: args.output.clone(),
            print_caught: args.caught,
            print_unviable: args.unviable,
            shuffle: !args.no_shuffle,
            show_times: !args.no_times,
            show_all_logs: args.all_logs,
            test_timeout: args
                .timeout
                .map(Duration::from_secs_f64)
                .unwrap_or(Duration::MAX),
        })
    }
}

fn build_glob_set(glob_set: &Vec<String>) -> Result<Option<GlobSet>> {
    if glob_set.is_empty() {
        return Ok(None);
    }

    let mut builder = GlobSetBuilder::new();
    for glob_str in glob_set {
        if glob_str.contains('/') || glob_str.contains(std::path::MAIN_SEPARATOR) {
            builder.add(Glob::new(glob_str)?);
        } else {
            builder.add(Glob::new(&format!("**/{}", glob_str))?);
        }
    }
    Ok(Some(builder.build()?))
}
