mod encoder;
mod photo;
mod recorder;

pub use photo::{photo_pixels, save_photo};
pub use recorder::Recorder;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn destination(output: &Path, extension: &str) -> Result<(tempfile::NamedTempFile, PathBuf), String> {
    std::fs::create_dir_all(output).map_err(|e| format!("Cannot create output directory: {e}"))?;
    let temporary = tempfile::Builder::new()
        .prefix(".khamura-")
        .suffix(&format!(".{extension}"))
        .tempfile_in(output)
        .map_err(|e| format!("Cannot create capture file: {e}"))?;
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let name = format!("khamura-{time}.{extension}");
    Ok((temporary, output.join(name)))
}

fn publish(temporary: tempfile::NamedTempFile, destination: PathBuf) -> Result<PathBuf, String> {
    temporary.as_file().sync_all().map_err(|e| format!("Cannot flush capture: {e}"))?;
    temporary.persist_noclobber(&destination).map_err(|e| format!("Cannot save capture: {e}"))?;
    Ok(destination)
}
