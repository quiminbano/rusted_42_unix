use crate::parser::ShellParser;
use std::fs::File;
use std::io::{self, Lines, StdinLock};
use std::process::Command;

pub struct Executer<'a> {
    input: &'a Vec<String>,
    infile_name: Option<String>,
    outfile_name: Option<String>,
    is_heredoc: bool,
    heredoc_content: Option<Result<String, String>>,
    main_commands: Option<Vec<String>>,
    arguments_commands: Option<Vec<Vec<String>>>,
    execution_commands: Option<Vec<Result<Command, String>>>, //Every command execution is a member of a vector
}

impl<'a> Executer<'a> {
    pub fn new(input: &'a Vec<String>) -> Executer {
        let inmutable_executer: Executer;
        let mut object = Executer {
            input,
            infile_name: None,
            outfile_name: None,
            is_heredoc: false,
            heredoc_content: None,
            main_commands: None,
            arguments_commands: None,
            execution_commands: None,
        };
        object.split_input();
        object.check_for_here_doc();
        object.check_for_infile();
        inmutable_executer = object;
        inmutable_executer
    }
    fn split_input(&self) {}
    fn check_for_here_doc(&mut self) {
        if self.input[0] == "here_doc" {
            self.is_heredoc = true;
            self.infile_name = Some("here_doc".to_owned());
            self.heredoc_content = self.fill_here_doc(&self.input[1]);
        }
    }
    fn fill_here_doc(&self, raw_delimiter: &String) -> Option<Result<String, String>> {
        let delimiter: String;
        let mut heredoc_temp: String;
        let mut line_temp: String;
        let stdin: io::Stdin;
        let mut lines: Lines<StdinLock<'static>>;

        heredoc_temp = String::new();
        delimiter = raw_delimiter.to_owned() + "\n";
        stdin = io::stdin();
        lines = stdin.lines();
        while let Some(result) = lines.next() {
            println!("rusted_pipex heredoc> ");
            match result {
                Ok(line) => line_temp = line,
                Err(_) => {
                    return Some(Err("Error processing here_doc".to_owned()));
                }
            };
            if line_temp == delimiter {
                break;
            }
            heredoc_temp += line_temp.as_str();
        }
        Some(Ok(heredoc_temp))
    }

    fn check_for_infile(&self) {
        if self.is_heredoc {
            return;
        }
    }
}
