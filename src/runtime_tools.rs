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
    Ffplay,
    Ffprobe,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Self::Pactl => "pactl",
            Self::Ffmpeg => "ffmpeg",
            Self::Ffplay => "ffplay",
            Self::Ffprobe => "ffprobe",
        }
    }
    fn variable(self) -> &'static str {
        match self {
            Self::Pactl => "KHAMURA_PACTL",
            Self::Ffmpeg => "KHAMURA_FFMPEG",
            Self::Ffplay => "KHAMURA_FFPLAY",
            Self::Ffprobe => "KHAMURA_FFPROBE",
        }
    }
    fn built_path(self) -> Option<&'static str> {
        match self {
            Self::Pactl => option_env!("KHAMURA_PACTL"),
            Self::Ffmpeg => option_env!("KHAMURA_FFMPEG"),
            Self::Ffplay => option_env!("KHAMURA_FFPLAY"),
            Self::Ffprobe => option_env!("KHAMURA_FFPROBE"),
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
    fn nix_helpers_launch_without_a_shell_path() {
        for tool in [Tool::Pactl, Tool::Ffmpeg] {
            if tool.built_path().is_none() {
                continue;
            }
            let output = command(tool)
                .env("PATH", "/nonexistent/khamura-bin")
                .arg(if matches!(tool, Tool::Pactl) {
                    "--version"
                } else {
                    "-version"
                })
                .output()
                .expect("embedded runtime tool must launch without PATH");
            assert!(output.status.success(), "{} failed", tool.name());
        }
    }

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
