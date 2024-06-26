#![allow(unused_imports)]
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Cursor, Read};
use anyhow::{bail, Context, Result};
use clap::Parser;
use tracing::debug;
use tracing_subscriber::FmtSubscriber;
use regex::Regex;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use transform_include::transformer::*;

//
// https://gcc.gnu.org/onlinedocs/cpp/Include-Syntax.html
//
// #include "file"
//
//   This variant is used for header files of your own program. It searches for
//   a file named `file` first in the directory containing the current file,
//   then in the quote directories and then the same directories used for
//   <file>.
//

#[derive(Debug, Parser)]
struct CliArgs {
    /// Don't write to disk, just print the diffs that would happen.
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Include path used when compiling. Can be specified multiple times.
    #[arg(short = 'I', long)]
    include: Vec<String>,

    /// Paths to map to other paths. Format: "/path/to/old:/path/to/new"
    #[arg(short, long, required = true)]
    map: Vec<String>,

    // /// Include files in the same directory explicitly, eg. "foo.h" -->
    // /// "./foo.h"
    // #[arg(long)]
    // explicit_same_dir: bool,

    /// Ignore unresolved include paths instead of exiting.
    #[arg(short, long)]
    keep_going: bool,

    /// File(s) to process
    #[arg(required = true)]
    file: Vec<String>,
}

#[derive(Debug)]
struct App {
    dry_run: bool,
    keep_going: bool,
    file: Vec<String>,
    transformer: Transformer,
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let cli_args = CliArgs::parse();
    // println!("{cli_args:#?}");

    let app = App::new(cli_args)?;
    // println!("{app:#?}");

    app.run()
}

impl App {
    fn new(cli_args: CliArgs) -> Result<Self> {
        let mut map = vec![];

        for x in cli_args.map {
            let Some((src, dst)) = x.split_once(':') else {
                bail!("bad map argument: {}", x);
            };

            map.push(MapFile {
                src: src.to_string(),
                dst: dst.to_string(),
            });
        }

        Ok(Self {
            dry_run: cli_args.dry_run,
            keep_going: cli_args.keep_going,
            file: cli_args.file,
            transformer: Transformer {
                keep_going: cli_args.keep_going,
                include: cli_args.include,
                map,
                checker: Box::new(FileExistsCheckImpl),
            },
        })
    }

    fn run(self) -> Result<()> {
        for path in self.file.iter() {
            debug!(file = ?path, "open");

            let content = std::fs
                ::read_to_string(path)
                .with_context(|| format!("failed to read file: {:?}", path))?;

            let cursor = Cursor::new(&content);

            let mut new_content = self.transformer
                .transform(cursor)
                .with_context(|| format!("failed to transform file: {:?}", path))?;

            new_content.push('\n');

            if self.dry_run {
                use similar::{ChangeTag, TextDiff};

                let diff = TextDiff::from_lines(&content, &new_content);

                for change in diff.iter_all_changes() {
                    let sign = match change.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal  => " ",
                    };

                    print!("{}{}", sign, change);
                }
            }

            else {
                std::fs
                    ::write(path, new_content)
                    .with_context(||
                        format!("failed to write file: {:?}", path)
                    )?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct FileExistsCheckImpl;

impl FileExistsCheck for FileExistsCheckImpl {
    fn file_exists(&self, path: &Path) -> Result<bool> {
        match std::fs::File::open(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}
