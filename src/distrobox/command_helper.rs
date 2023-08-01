use crate::utils::command_helper::*;

pub fn distrobox_assemble(
    assemble_file_path: &str,
    action: &str,
    args: &[&str],
) -> Result<(String, String), CommandError> {
    let mut command_args = vec!["assemble", action, "--file", assemble_file_path];
    command_args.extend_from_slice(args);
    let (stdout, stderr) = run_command("distrobox", &command_args)?;
    Ok((stdout, stderr))
}

pub fn distrobox_enter(
    container_name: &str,
    args: &[&str],
    run_cmds: &[&str],
) -> Result<(String, String), CommandError> {
    let mut command_args = vec!["enter","--name", container_name];
    command_args.extend_from_slice(args);
    command_args.push("--");
    command_args.extend(run_cmds);
    let (stdout, stderr) = run_command_no_pipe("distrobox", &command_args)?;
    Ok((stdout, stderr))
}
