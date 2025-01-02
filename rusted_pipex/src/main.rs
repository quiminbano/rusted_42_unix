use std::env;
use std::process::ExitCode;
mod executer;
use crate::executer::Executer;

fn main() -> ExitCode {
    let mut args: Vec<String>;
    let exit_status: u8;
    let mut commands_executer: Executer;

    args = env::args().collect();
    args.remove(0);
    match Executer::new(&args) {
        Ok(executer) => commands_executer = executer,
        Err(_) => {
            eprintln!("rusted_pipex: Invalid ammount of arguments");
            return ExitCode::FAILURE;
        }
    };
    commands_executer.iterate_commands();
    exit_status = commands_executer.get_exit_status();
    drop(commands_executer);
    ExitCode::from(exit_status)
}
