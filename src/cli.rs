use std::ffi::OsString;

const HELP: &str = "Khamura — a minimal camera app

Usage: khamura [OPTION]

  --install-desktop  Install the current user's launcher and icon (no camera access)
  -h, --help         Show this help
  -V, --version      Show the version

Run without options to open the camera.";

#[derive(Debug, PartialEq)]
enum Action {
    Launch,
    InstallDesktop,
    Help,
    Version,
}

fn parse(arguments: &[OsString]) -> Result<Action, &'static str> {
    match arguments {
        [] => Ok(Action::Launch),
        [argument] => match argument.to_str() {
            Some("--install-desktop") => Ok(Action::InstallDesktop),
            Some("--help" | "-h") => Ok(Action::Help),
            Some("--version" | "-V") => Ok(Action::Version),
            _ => Err("Unknown option. Use khamura --help for usage."),
        },
        _ => Err("Expected at most one option. Use khamura --help for usage."),
    }
}

/// Handle non-GUI commands before reading config or initializing GPUI/devices.
/// None means launch normally; Some is the command's process exit status.
pub fn handle() -> Option<i32> {
    match parse(&std::env::args_os().skip(1).collect::<Vec<_>>()) {
        Ok(Action::Launch) => None,
        Ok(Action::Help) => {
            println!("{HELP}");
            Some(0)
        }
        Ok(Action::Version) => {
            println!("khamura {}", env!("CARGO_PKG_VERSION"));
            Some(0)
        }
        Ok(Action::InstallDesktop) => Some(match crate::desktop::install() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("{error}");
                1
            }
        }),
        Err(error) => {
            eprintln!("{error}");
            Some(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_explicit_install_option_changes_desktop_files() {
        assert_eq!(parse(&[]).unwrap(), Action::Launch);
        assert_eq!(
            parse(&["--install-desktop".into()]).unwrap(),
            Action::InstallDesktop
        );
        assert_eq!(parse(&["--help".into()]).unwrap(), Action::Help);
        assert_eq!(parse(&["--version".into()]).unwrap(), Action::Version);
        assert!(parse(&["--unknown".into()]).is_err());
        assert!(parse(&["--install-desktop".into(), "extra".into()]).is_err());
    }
}
