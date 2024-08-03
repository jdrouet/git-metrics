use lcov::{Reader, Record};
use std::path::PathBuf;

use crate::entity::metric::Metric;

#[derive(Debug)]
struct LcovFile {
    options: LcovImporterOptions,
    branches_count: usize,
    branches_hit: usize,
    functions_count: usize,
    functions_hit: usize,
    lines_count: usize,
    lines_hit: usize,
}

impl LcovFile {
    fn handle(&mut self, record: lcov::Record) {
        match record {
            Record::BranchesFound { found } => {
                self.branches_count += found as usize;
            }
            Record::BranchesHit { hit } => {
                self.branches_hit += hit as usize;
            }
            Record::FunctionsFound { found } => {
                self.functions_count += found as usize;
            }
            Record::FunctionsHit { hit } => {
                self.functions_hit += hit as usize;
            }
            Record::LinesFound { found } => {
                self.lines_count += found as usize;
            }
            Record::LinesHit { hit } => {
                self.lines_hit += hit as usize;
            }
            _ => {}
        }
    }

    fn new(path: PathBuf, options: LcovImporterOptions) -> Result<Self, super::Error> {
        let mut res = Self {
            options,
            branches_count: 0,
            branches_hit: 0,
            functions_count: 0,
            functions_hit: 0,
            lines_count: 0,
            lines_hit: 0,
        };
        for item in Reader::open_file(path)? {
            match item {
                Ok(record) => res.handle(record),
                Err(error) => {
                    return Err(super::Error::InvalidFormat {
                        source: Box::new(error),
                    })
                }
            }
        }
        Ok(res)
    }

    fn build(self) -> Vec<Metric> {
        let mut res = Vec::with_capacity(self.options.expected_count());
        // branches
        if self.options.branches {
            let branches_count = self.branches_count as f64;
            res.push(Metric::new("coverage.branches.count", branches_count));
            let branches_hit = self.branches_hit as f64;
            res.push(Metric::new("coverage.branches.hit", branches_hit));
            if self.branches_count > 0 {
                let branches_percentage = branches_hit / branches_count;
                res.push(Metric::new(
                    "coverage.branches.percentage",
                    branches_percentage,
                ));
            }
        } else {
            tracing::debug!("skipping collecting branches");
        }
        // functions
        if self.options.functions {
            let functions_count = self.functions_count as f64;
            res.push(Metric::new("coverage.functions.count", functions_count));
            let functions_hit = self.functions_hit as f64;
            res.push(Metric::new("coverage.functions.hit", functions_hit));
            if self.functions_count > 0 {
                let functions_percentage = functions_hit / functions_count;
                res.push(Metric::new(
                    "coverage.functions.percentage",
                    functions_percentage,
                ));
            }
        } else {
            tracing::debug!("skipping collecting functions");
        }
        // lines
        if self.options.lines {
            let lines_count = self.lines_count as f64;
            res.push(Metric::new("coverage.lines.count", lines_count));
            let lines_hit = self.lines_hit as f64;
            res.push(Metric::new("coverage.lines.hit", lines_hit));
            if self.lines_count > 0 {
                let lines_percentage = lines_hit / lines_count;
                res.push(Metric::new("coverage.lines.percentage", lines_percentage));
            }
        } else {
            tracing::debug!("skipping collecting lines");
        }
        //
        res
    }
}

#[derive(Debug)]
pub(crate) struct LcovImporterOptions {
    pub branches: bool,
    pub functions: bool,
    pub lines: bool,
}

impl LcovImporterOptions {
    fn expected_count(&self) -> usize {
        let mut count = 0;
        if self.branches {
            count += 3;
        }
        if self.functions {
            count += 3;
        }
        if self.lines {
            count += 3;
        }
        count
    }
}

#[derive(Debug)]
pub(crate) struct LcovImporter {
    pub path: PathBuf,
    pub options: LcovImporterOptions,
}

impl LcovImporter {
    #[inline(always)]
    pub(crate) fn new(path: PathBuf, options: LcovImporterOptions) -> Self {
        Self { path, options }
    }
}

impl super::Importer for LcovImporter {
    fn import(self) -> Result<Vec<crate::entity::metric::Metric>, super::Error> {
        LcovFile::new(self.path, self.options).map(|file| file.build())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::importer::{
        lcov::{LcovImporter, LcovImporterOptions},
        Importer,
    };

    #[test]
    fn should_load_metrics_complete() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("asset")
            .join("lcov.info");
        let importer = LcovImporter::new(
            file,
            LcovImporterOptions {
                branches: true,
                functions: true,
                lines: true,
            },
        );
        let metrics = importer.import().unwrap();
        assert_eq!(metrics.len(), 8);
        assert_eq!(metrics[0].header.name, "coverage.branches.count");
        assert_eq!(metrics[0].value, 0.0);
        assert_eq!(metrics[1].header.name, "coverage.branches.hit");
        assert_eq!(metrics[1].value, 0.0);
        assert_eq!(metrics[2].header.name, "coverage.functions.count");
        assert_eq!(metrics[2].value, 338.0);
        assert_eq!(metrics[3].header.name, "coverage.functions.hit");
        assert_eq!(metrics[3].value, 255.0);
        assert_eq!(metrics[4].header.name, "coverage.functions.percentage");
        assert_eq!(metrics[4].value, 255.0 / 338.0);
        assert_eq!(metrics[5].header.name, "coverage.lines.count");
        assert_eq!(metrics[5].value, 2721.0);
        assert_eq!(metrics[6].header.name, "coverage.lines.hit");
        assert_eq!(metrics[6].value, 2298.0);
        assert_eq!(metrics[7].header.name, "coverage.lines.percentage");
        assert_eq!(metrics[7].value, 2298.0 / 2721.0);
    }

    #[test]
    fn should_load_metrics_without_branches() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("asset")
            .join("lcov.info");
        let importer = LcovImporter::new(
            file,
            LcovImporterOptions {
                branches: false,
                functions: true,
                lines: true,
            },
        );
        let metrics = importer.import().unwrap();
        assert_eq!(metrics.len(), 6);
        assert_eq!(metrics[0].header.name, "coverage.functions.count");
        assert_eq!(metrics[1].header.name, "coverage.functions.hit");
        assert_eq!(metrics[2].header.name, "coverage.functions.percentage");
        assert_eq!(metrics[3].header.name, "coverage.lines.count");
        assert_eq!(metrics[4].header.name, "coverage.lines.hit");
        assert_eq!(metrics[5].header.name, "coverage.lines.percentage");
    }

    #[test]
    fn should_load_metrics_without_functions() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("asset")
            .join("lcov.info");
        let importer = LcovImporter::new(
            file,
            LcovImporterOptions {
                branches: true,
                functions: false,
                lines: true,
            },
        );
        let metrics = importer.import().unwrap();
        assert_eq!(metrics.len(), 5);
        assert_eq!(metrics[0].header.name, "coverage.branches.count");
        assert_eq!(metrics[1].header.name, "coverage.branches.hit");
        assert_eq!(metrics[2].header.name, "coverage.lines.count");
        assert_eq!(metrics[3].header.name, "coverage.lines.hit");
        assert_eq!(metrics[4].header.name, "coverage.lines.percentage");
    }

    #[test]
    fn should_load_metrics_without_lines() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("asset")
            .join("lcov.info");
        let importer = LcovImporter::new(
            file,
            LcovImporterOptions {
                branches: true,
                functions: true,
                lines: false,
            },
        );
        let metrics = importer.import().unwrap();
        assert_eq!(metrics.len(), 5);
        assert_eq!(metrics[0].header.name, "coverage.branches.count");
        assert_eq!(metrics[1].header.name, "coverage.branches.hit");
        assert_eq!(metrics[2].header.name, "coverage.functions.count");
        assert_eq!(metrics[3].header.name, "coverage.functions.hit");
        assert_eq!(metrics[4].header.name, "coverage.functions.percentage");
    }

    #[test]
    #[should_panic = "Io { source: Os { code: 2, kind: NotFound, message: \"No such file or directory\" } }"]
    fn should_fail_opening() {
        let file = PathBuf::default().join("nowhere").join("lcov.info");
        let importer = LcovImporter::new(
            file,
            LcovImporterOptions {
                branches: true,
                functions: true,
                lines: true,
            },
        );
        let _ = importer.import().unwrap();
    }
}
