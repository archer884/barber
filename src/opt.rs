use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Primary directory for any duplicates located.
    #[structopt(parse(from_os_str))]
    target: PathBuf,

    /// Context within which duplicates will be sought.
    #[structopt(parse(from_os_str))]
    context: PathBuf,

    /// Remove duplicates.
    #[structopt(short = "f", long = "force")]
    pub force: bool,

    /// Do not report deleted files.
    #[structopt(short = "s", long = "silent")]
    pub silent: bool,
}

impl Opt {
    pub fn from_args() -> Self {
        StructOpt::from_args()
    }

    pub fn target(&self) -> &Path {
        &self.target
    }

    pub fn context(&self) -> &Path {
        &self.context
    }
}
