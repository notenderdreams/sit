use crate::error::Result;
use crate::git;

pub fn show_log() -> Result<()> {
    git::log_graph()?;
    Ok(())
}
