//! Runtime helpers use Nix-provided absolute paths when built in the dev shell.
//! This also supports launching the resulting binary from a desktop or plain shell.
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy)]
pub enum Tool {
    Pactl,
    Ffmpeg,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Self::Pactl => "pactl",
            Self::Ffmpeg => "ffmpeg",
        }
    }
    fn variable(self) -> &'static str {
        match self {
            Self::Pactl => "KHAMURA_PACTL",
            Self::Ffmpeg => "KHAMURA_FFMPEG",
        }
    }
    fn built_path(self) -> Option<&'static str> {
        match self {
            Self::Pactl => option_env!("KHAMURA_PACTL"),
            Self::Ffmpeg => option_env!("KHAMURA_FFMPEG"),
        }
    }
}

pub fn command(tool: Tool) -> Command {
    let path = std::env::var_os(tool.variable())
        .map(PathBuf::from)
        .unwrap_or_else(|| resolve_built_path(tool.built_path(), tool.name()));
    Command::new(path)
}

fn resolve_built_path(built: Option<&str>, name: &str) -> PathBuf {
    built
        .filter(|path| Path::new(path).is_file())
        .map(PathBuf::from)
        .unwrap_or_else(|| name.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uses_embedded_tool_path_without_relying_on_launch_path() {
        let file = tempfile::NamedTempFile::new().unwrap();
        assert_eq!(
            resolve_built_path(file.path().to_str(), "pactl"),
            file.path()
        );
        assert_eq!(
            resolve_built_path(Some("/nonexistent/khamura/pactl"), "pactl"),
            PathBuf::from("pactl")
        );
        assert_eq!(resolve_built_path(None, "pactl"), PathBuf::from("pactl"));
    }
}
