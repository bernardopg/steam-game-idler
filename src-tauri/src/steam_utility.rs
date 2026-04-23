use std::path::{Path, PathBuf};

pub const STEAM_UTILITY_PATH_ENV: &str = "SGI_STEAM_UTILITY_PATH";

pub fn steam_utility_filename_for_os(target_os: &str) -> &'static str {
    match target_os {
        "windows" => "SteamUtility.exe",
        _ => "SteamUtility.Cli",
    }
}

pub fn default_steam_utility_path_from_base(base_dir: &Path, target_os: &str) -> PathBuf {
    base_dir
        .join("libs")
        .join(steam_utility_filename_for_os(target_os))
}

pub fn resolve_steam_utility_path_from_base(base_dir: &Path) -> PathBuf {
    if let Ok(override_path) = std::env::var(STEAM_UTILITY_PATH_ENV) {
        if !override_path.trim().is_empty() {
            return PathBuf::from(override_path);
        }
    }

    default_steam_utility_path_from_base(base_dir, std::env::consts::OS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn uses_windows_filename_for_windows_targets() {
        assert_eq!(steam_utility_filename_for_os("windows"), "SteamUtility.exe");
    }

    #[test]
    fn uses_plain_filename_for_non_windows_targets() {
        assert_eq!(steam_utility_filename_for_os("linux"), "SteamUtility.Cli");
        assert_eq!(steam_utility_filename_for_os("macos"), "SteamUtility.Cli");
    }

    #[test]
    fn builds_default_path_from_base_directory() {
        let base_dir = Path::new("/tmp/sgi");
        let resolved = default_steam_utility_path_from_base(base_dir, "linux");

        assert_eq!(resolved, Path::new("/tmp/sgi/libs/SteamUtility.Cli"));
    }
}
