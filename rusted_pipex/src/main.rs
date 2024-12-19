use std::env;
use std::process::ExitCode;
//mod executer;
mod parser;
use crate::parser::ShellParser;
//use crate::executer::Executer;

fn main() -> ExitCode {
    let args: Vec<String>;
    let test_parser: Vec<String>;
    //let commands_executer: Executer;

    args = env::args().collect();
    if args.len() < 5 {
        eprintln!("rusted_pipex: Invalid ammount of arguments");
        return ExitCode::FAILURE;
    }
    //commands_executer = Executer::new(&args);
    test_parser = ShellParser::parsed_input(&args[1]);
    for stringi in test_parser.iter() {
        println!("This is the string: {stringi}");
    }
    ExitCode::SUCCESS
}
