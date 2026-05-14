use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub fn apply_hidden_command_style(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }

    command
}

pub fn spawn_url(url: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", url]);
        apply_hidden_command_style(&mut command)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        spawn_first_available(&[
            ("xdg-open", &[url][..]),
            ("gio", &["open", url][..]),
            ("kde-open5", &[url][..]),
            ("gnome-open", &[url][..]),
        ])
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn spawn_path(path: &std::path::Path) -> Result<(), String> {
    let path_arg = path.to_string_lossy().to_string();
    spawn_first_available(&[
        ("xdg-open", &[path_arg.as_str()][..]),
        ("gio", &["open", path_arg.as_str()][..]),
        ("kde-open5", &[path_arg.as_str()][..]),
        ("gnome-open", &[path_arg.as_str()][..]),
    ])
}

#[cfg(all(unix, not(target_os = "macos")))]
fn spawn_first_available(candidates: &[(&str, &[&str])]) -> Result<(), String> {
    let mut errors = Vec::new();

    for (program, args) in candidates {
        match Command::new(program).args(*args).spawn() {
            Ok(_) => return Ok(()),
            Err(e) => errors.push(format!("{}: {}", program, e)),
        }
    }

    Err(format!(
        "Failed to open target with any known Linux opener ({})",
        errors.join("; ")
    ))
}
