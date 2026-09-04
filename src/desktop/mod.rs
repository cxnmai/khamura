mod entry;

use std::{
    env, fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

const ICON: &[u8] = include_bytes!("../../assets/khamura.svg");

pub fn install() -> Result<(), String> {
    let data = data_directory(
        env::var_os("XDG_DATA_HOME").as_deref(),
        env::var_os("HOME").as_deref(),
    )?;
    let executable =
        env::current_exe().map_err(|error| format!("Cannot locate Khamura: {error}"))?;
    let (launcher, icon) = install_at(&data, &executable)?;
    println!(
        "Installed launcher: {}\nInstalled icon: {}",
        launcher.display(),
        icon.display()
    );
    println!(
        "Launcher uses {}. Run --install-desktop again if you move the binary.",
        executable.display()
    );
    Ok(())
}

fn data_directory(
    xdg: Option<&std::ffi::OsStr>,
    home: Option<&std::ffi::OsStr>,
) -> Result<PathBuf, String> {
    if let Some(path) = xdg.map(Path::new).filter(|path| path.is_absolute()) {
        return Ok(path.to_path_buf());
    }
    home.map(Path::new)
        .filter(|path| path.is_absolute())
        .map(|path| path.join(".local/share"))
        .ok_or_else(|| {
            "Set an absolute XDG_DATA_HOME or HOME to install desktop integration".into()
        })
}

fn install_at(data: &Path, executable: &Path) -> Result<(PathBuf, PathBuf), String> {
    let contents = entry::entry(executable)?;
    let launcher = data.join("applications/khamura.desktop");
    let icon = data.join("icons/hicolor/scalable/apps/khamura.svg");
    // Publish the icon before the launcher that references it. Re-running is safe.
    write_atomic(&icon, ICON)?;
    write_atomic(&launcher, contents.as_bytes())?;
    Ok((launcher, icon))
}

fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    let result = (|| -> std::io::Result<()> {
        let parent = path.parent().expect("desktop destinations have a parent");
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(contents)?;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o644))?;
        temporary.as_file().sync_all()?;
        temporary.persist(path).map_err(|error| error.error)?;
        Ok(())
    })();
    result.map_err(|error| format!("Cannot install {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follows_xdg_data_directory_defaults_without_mutating_environment() {
        assert_eq!(
            data_directory(Some("/custom/data".as_ref()), None).unwrap(),
            Path::new("/custom/data")
        );
        for xdg in [None, Some("".as_ref()), Some("relative".as_ref())] {
            assert_eq!(
                data_directory(xdg, Some("/home/test".as_ref())).unwrap(),
                Path::new("/home/test/.local/share")
            );
        }
        assert!(data_directory(None, None).is_err());
        assert!(data_directory(None, Some("relative".as_ref())).is_err());
    }

    #[test]
    fn installs_embedded_icon_and_launcher_and_updates_existing_installation() {
        let directory = tempfile::tempdir().unwrap();
        let (launcher, icon) = install_at(directory.path(), Path::new("/old/bin/khamura")).unwrap();
        assert_eq!(fs::read(&icon).unwrap(), ICON);
        assert_eq!(
            fs::metadata(&launcher).unwrap().permissions().mode() & 0o777,
            0o644
        );
        install_at(directory.path(), Path::new("/new path/khamura")).unwrap();
        let contents = fs::read_to_string(launcher).unwrap();
        assert!(contents.contains("Exec=\"/new path/khamura\""));
        assert!(!contents.contains("/old/bin/"));
    }
}
