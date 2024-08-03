/// Imports metrics from a lcov.info file
///
/// This can be obtained with the following commands
///
/// For Rust, use <https://github.com/taiki-e/cargo-llvm-cov> with the following command.
///
///     cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
///
/// For other languages, feel free to open a PR or an issue with the command.
#[derive(clap::Parser, Debug)]
pub(super) struct LcovImporter {
    /// Path to the lcov.info file
    path: std::path::PathBuf,
    /// Skip importing branch coverage
    #[clap(long, default_value = "false")]
    disable_branches: bool,
    /// Skip importing function coverage
    #[clap(long, default_value = "false")]
    disable_functions: bool,
    /// Skip importing line coverage
    #[clap(long, default_value = "false")]
    disable_lines: bool,
}

impl LcovImporter {
    #[inline(always)]
    fn options(&self) -> crate::importer::lcov::LcovImporterOptions {
        crate::importer::lcov::LcovImporterOptions {
            branches: !self.disable_branches,
            functions: !self.disable_functions,
            lines: !self.disable_lines,
        }
    }
}

impl crate::importer::Importer for LcovImporter {
    fn import(self) -> Result<Vec<crate::entity::metric::Metric>, crate::importer::Error> {
        let opts = self.options();
        crate::importer::lcov::LcovImporter::new(self.path, opts).import()
    }
}
