use std::path::Path;

/// Quote both the desktop string value and the Exec argument. No shell is used.
/// https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
pub(super) fn entry(executable: &Path) -> Result<String, String> {
    if !executable.is_absolute() {
        return Err("The launcher requires an absolute executable path".into());
    }
    let path = executable
        .to_str()
        .ok_or("Executable path must be valid UTF-8")?;
    if path.chars().any(|c| c.is_control() || c == '=') {
        return Err("Executable path contains characters unsupported by desktop launchers".into());
    }
    let mut quoted = String::from("\"");
    for character in path.chars() {
        match character {
            '\\' | '"' | '`' | '$' => {
                quoted.push('\\');
                quoted.push(character);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    let command = quoted.replace('\\', "\\\\");
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=Khamura\nComment=Take photos and record videos\nExec={command}\nIcon=khamura\nTerminal=false\nCategories=AudioVideo;Video;\nKeywords=camera;photo;video;webcam;\nStartupWMClass=khamura\n"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_quotes_paths_and_literal_field_codes() {
        let actual = entry(Path::new("/home/test user/100%/khamura")).unwrap();
        assert!(actual.contains("Exec=\"/home/test user/100%%/khamura\"\n"));
        let escaped = entry(Path::new("/tmp/a\\b\"$`/khamura")).unwrap();
        let command = escaped
            .lines()
            .find(|line| line.starts_with("Exec="))
            .unwrap();
        assert_eq!(command, r#"Exec="/tmp/a\\\\b\\"\\$\\`/khamura""#);
        assert!(actual.contains("Icon=khamura\n"));
    }

    #[test]
    fn invalid_paths_cannot_inject_desktop_fields() {
        for path in ["relative/khamura", "/tmp/bad\nTerminal=true", "/tmp/a=b"] {
            assert!(entry(Path::new(path)).is_err());
        }
    }
}
