use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

use case::CaseExt;

use crate::generator::generate;
use crate::parser::parse;

pub struct Config {
    out_dir: PathBuf,
}

impl Config {
    pub fn new() -> Self {
        Config {
            out_dir: std::env::var("OUT_DIR").unwrap().into(),
        }
    }

    pub fn out_dir(self, path: impl Into<PathBuf>) -> Self {
        Self {
            out_dir: path.into(),
        }
    }

    pub fn compile<P: AsRef<Path>>(self, files: &[P]) -> Result<()> {
        for file in files {
            let source = std::fs::read_to_string(file)?;
            let colfer =
                parse(&source).map_err(|err| Error::new(ErrorKind::Other, err.to_string()))?;

            std::fs::write(
                self.out_dir
                    .join(colfer.package.to_snake())
                    .with_extension("rs"),
                generate(&colfer),
            )?;
        }

        Ok(())
    }
}
