#![allow(unused_imports)]
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Cursor, Read};
use anyhow::{bail, Result};
use clap::Parser;
use tracing::{debug, warn};
use tracing_subscriber::FmtSubscriber;
use regex::Regex;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Transformer {
    pub keep_going: bool,
    pub include: Vec<String>,
    pub map: Vec<MapFile>,
    pub checker: Box<dyn FileExistsCheck>,
}

#[derive(Debug)]
pub struct MapFile {
    pub src: String,
    pub dst: String,
}

pub trait FileExistsCheck: std::fmt::Debug {
    fn file_exists(&self, path: &Path) -> Result<bool>;
}

impl Transformer {
    pub fn transform(&self, r: impl Read) -> Result<String> {
        static RE: Lazy<Regex> = Lazy::new(||
            Regex::new(r#"^(\s*#include\s*")([^"]+)(".*)$"#).unwrap()
        );

        let mut new_lines = vec![];

        for line in BufReader::new(r).lines() {
            let line = line?;

            let Some(captures) = RE.captures(&line) else {
                new_lines.push(line);
                continue;
            };

            debug!(line, "match");

            let path = captures.get(2).unwrap().as_str();
            let new_path = match self.transform_path(path) {
                Ok(x) => x,
                Err(e) => {
                    if self.keep_going {
                        new_lines.push(line);
                        continue;
                    }

                    bail!(e);
                },
            };

            let new_line = format!(
                "{}{}{}",
                captures.get(1).unwrap().as_str(),
                new_path,
                captures.get(3).unwrap().as_str()
            );

            debug!(line = new_line, "into ");

            new_lines.push(new_line);
        }

        Ok(new_lines.join("\n"))
    }

    fn transform_path(&self, path: &str) -> Result<String> {
        let resolved_path = self.resolve_path(path)?;
        let mut first_match = None;
        let mut other_matches = vec![];

        for map in self.map.iter() {
            if let Some(suffix) = resolved_path.strip_prefix(&map.src) {
                let value = format!("{}{}", map.dst, suffix);

                if first_match.is_none() {
                    first_match = Some(value);
                }

                else {
                    other_matches.push(value);
                }
            }
        }

        let Some(first) = first_match else {
            return Ok(path.to_string()); // no mapping --> no transform
        };

        if !other_matches.is_empty() {
            warn!(?first, other = ?other_matches, "multiple candidates");
        }

        Ok(first)
    }

    fn resolve_path(&self, path: &str) -> Result<String> {
        for include_path in self.include.iter() {
            let path_candidate = PathBuf::from(include_path).join(path);

            if self.checker.file_exists(&path_candidate)? {
                return Ok(path_candidate.to_string_lossy().to_string());
            }
        }

        bail!("failed to transform path: {:?}", path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_tracing() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    }


    #[derive(Debug)]
    struct FileExistsCheckFake {
        files: Vec<PathBuf>,
    }

    impl FileExistsCheck for FileExistsCheckFake {
        fn file_exists(&self, path: &Path) -> Result<bool> {
            for i in self.files.iter() {
                if i == path {
                    return Ok(true);
                }
            }

            Ok(false)
        }
    }

    #[test]
    fn resolves_and_maps() -> Result<()> {
        with_tracing();

        let content = r#"
            // comment
            #include "foo.h"
            #include "sub/bar.h"

            #ifdef WHATEVER
                #include "baz.h"
            #endif

            int main(void) { return 0; }
        "#;

        let expected = r#"
            // comment
            #include "project/a/foo.h"
            #include "project/a/sub/bar.h"

            #ifdef WHATEVER
                #include "project/b/baz.h"
            #endif

            int main(void) { return 0; }
        "#;

        let transformer = Transformer {
            keep_going: true,
            include: vec![
                "/path/to/project/a".to_string(),
                "/path/to/project/b".to_string(),
            ],
            map: vec![
                MapFile {
                    src: "/path/to/project/a".to_string(),
                    dst: "project/a".to_string()
                },
                MapFile {
                    src: "/path/to/project/b".to_string(),
                    dst: "project/b".to_string()
                },
            ],
            checker: Box::new(FileExistsCheckFake {
                files: vec![
                    "/path/to/project/a/foo.h".into(),
                    "/path/to/project/a/sub/bar.h".into(),
                    "/path/to/project/b/baz.h".into(),
                ],
            }),
        };

        let r = Cursor::new(content);
        let actual = transformer.transform(r)?;

        similar_asserts::assert_eq!(actual, expected);

        Ok(())
    }
}
