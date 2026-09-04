// Keep labels compact in the 280px settings panel; full paths live in tooltips.
const MAX_PATH_CHARS: usize = 32;

pub(super) fn path_label(path: &str) -> String {
    let chars: Vec<char> = path.chars().collect();
    if chars.len() <= MAX_PATH_CHARS {
        return path.to_owned();
    }
    let prefix = (MAX_PATH_CHARS - 1) / 2;
    let suffix = MAX_PATH_CHARS - prefix - 1;
    chars[..prefix]
        .iter()
        .copied()
        .chain(std::iter::once('…'))
        .chain(chars[chars.len() - suffix..].iter().copied())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_short_paths_and_both_ends_of_long_paths() {
        let short = "/home/chinmay/Pictures/khamura";
        assert_eq!(path_label(short), short);
        let long = "/home/chinmay/.config/khamura/config.toml";
        let label = path_label(long);
        assert_eq!(label.chars().count(), MAX_PATH_CHARS);
        assert!(label.starts_with("/home/chinmay/"));
        assert!(label.ends_with("config.toml"));
        assert!(label.contains('…'));
        assert_eq!(path_label(&"界".repeat(40)).chars().count(), MAX_PATH_CHARS);
    }
}
