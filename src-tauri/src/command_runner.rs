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
