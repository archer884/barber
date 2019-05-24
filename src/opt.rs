use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Primary directory for any duplicates located.
    #[structopt(parse(from_os_str))]
    target: PathBuf,

    /// Context within which duplicates will be sought.
    #[structopt(parse(from_os_str))]
    context: Option<PathBuf>,

    /// Remove duplicates.
    #[structopt(short = "f", long = "force")]
    pub force: bool,

    /// Do not report deleted files.
    #[structopt(short = "s", long = "silent")]
    pub silent: bool,

    /// Debug delete process.
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,
}

impl Opt {
    pub fn from_args() -> Self {
        StructOpt::from_args()
    }

    pub fn target(&self) -> &Path {
        &self.target
    }

    pub fn context(&self) -> Option<&Path> {
        self.context.as_ref().map(AsRef::as_ref)
    }
}
