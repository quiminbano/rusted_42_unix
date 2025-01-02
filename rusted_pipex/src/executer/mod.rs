mod parser;
use parser::ShellParser;
use std::fs::File;
use std::io::{self, Lines, StdinLock, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};

pub struct Executer<'a> {
    input: &'a Vec<String>,
    infile_name: String,
    infile_content: Result<File, String>,
    infile_error: bool,
    heredoc_content: Result<String, String>,
    has_heredoc: bool,
    outfile_name: String,
    outfile_content: Result<File, String>,
    outfile_error: bool,
    main_commands: Vec<String>,
    arguments_commands: Vec<Vec<String>>,
    ammout_commands: usize,
    execution_commands: Vec<Option<Child>>, //Every child process is a member of a vector
    exit_status: i32,
}

impl<'a> Executer<'a> {
    pub fn new(input: &'a Vec<String>) -> Result<Executer, String> {
        let inmutable_executer: Executer;
        let mut object: Executer;
        let outfile_index: usize;
        let mut splitted_input: Vec<Vec<String>>;
        let temp_infile: Result<File, String>;

        object = Executer {
            input,
            infile_name: String::new(),
            infile_content: Err(String::new()),
            infile_error: false,
            heredoc_content: Err(String::new()),
            has_heredoc: false,
            outfile_name: String::new(),
            outfile_content: Err(String::new()),
            outfile_error: false,
            main_commands: Vec::new(),
            arguments_commands: Vec::new(),
            ammout_commands: 0,
            execution_commands: Vec::new(),
            exit_status: 0,
        };
        if input.len() < 4 {
            return Err("".to_owned());
        }
        splitted_input = object.split_input();
        if splitted_input[0][0] == "here_doc" && splitted_input.len() < 5 {
            return Err("".to_owned());
        }
        (object.infile_name, temp_infile) = object.analyze_file(&mut splitted_input, 0);
        object.check_input_type(temp_infile, &input[1], &mut splitted_input);
        outfile_index = splitted_input.len() - 1;
        (object.outfile_name, object.outfile_content) =
            object.analyze_file(&mut splitted_input, outfile_index);
        object.ammout_commands = splitted_input.len();
        object.classify_commands_and_arguments(splitted_input); //At this point, when this function finish its execution, splitted_input is stored in object.arguments_commands
        object.command_exec_prep();
        inmutable_executer = object;
        Ok(inmutable_executer)
    }

    fn split_input(&mut self) -> Vec<Vec<String>> {
        let mut temp_vec: Vec<String>;
        let mut parsed_input: Vec<Vec<String>>;

        parsed_input = Vec::new();
        for string in self.input.iter() {
            temp_vec = ShellParser::parsed_input(&string);
            parsed_input.push(temp_vec);
        }
        parsed_input
    }

    fn analyze_file(
        &self,
        arguments: &mut Vec<Vec<String>>,
        index: usize,
    ) -> (String, Result<File, String>) {
        let file_name: String;
        let file_to_open: Result<File, String>;
        let type_of_file: &str;

        type_of_file = if index == 0 { "infile" } else { "outfile" };
        if arguments[index].len() != 1 {
            file_name = format!("Invalid format name for {}", type_of_file);
            arguments.remove(index);
            return (
                file_name,
                Err(format!(
                    "rusted_pipex could not process {} name provided",
                    type_of_file
                )
                .to_owned()),
            );
        }
        file_name = arguments[index][0].clone();
        arguments.remove(index);
        if index == 0 {
            file_to_open = match File::open(&file_name) {
                Ok(file) => Ok(file),
                Err(error_message) => Err(error_message.to_string()),
            };
        } else {
            file_to_open = match File::create(&file_name) {
                Ok(file) => Ok(file),
                Err(error_message) => Err(error_message.to_string()),
            };
        }
        (file_name, file_to_open)
    }

    fn check_input_type(
        &mut self,
        infile_content: Result<File, String>,
        delimiter: &String,
        arguments: &mut Vec<Vec<String>>,
    ) {
        if self.infile_name == "here_doc" {
            arguments.remove(0);
            self.has_heredoc = true;
            self.heredoc_content = self.fill_here_doc(delimiter);
            return;
        }
        self.infile_content = infile_content;
    }

    fn fill_here_doc(&self, raw_delimiter: &String) -> Result<String, String> {
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
                    return Err("Error processing here_doc".to_owned());
                }
            };
            if line_temp == delimiter {
                break;
            }
            heredoc_temp += line_temp.as_str();
        }
        Ok(heredoc_temp)
    }

    fn classify_commands_and_arguments(&mut self, mut arguments: Vec<Vec<String>>) {
        let mut index: usize;

        index = 0;
        while index < self.ammout_commands {
            self.main_commands.push(arguments[index][0].clone());
            arguments[index].remove(0);
            index += 1;
        }
        self.arguments_commands = arguments;
    }

    fn command_exec_prep(&mut self) {
        let mut current_command: Command;
        let mut current_child: Result<Child, io::Error>;
        let mut index: usize;

        index = 0;
        self.print_files_error();

        while index < self.ammout_commands {
            if (index == 0 && self.infile_error)
                || (index == self.ammout_commands - 1 && self.outfile_error)
            {
                index += 1;
                self.execution_commands.push(None);
                continue;
            }
            current_command = Command::new(&self.main_commands[index]);
            if self.arguments_commands[index].len() != 0 {
                current_command.args(&self.arguments_commands[index]);
            }
            self.handle_redirections(&mut current_command, index);
            current_child = current_command.spawn();
            match current_child {
                Ok(mut child) => {
                    if index == 0 && self.has_heredoc {
                        self.fill_stdin_heredoc(&mut child);
                    }
                    self.execution_commands.push(Some(child));
                }
                Err(message) => {
                    self.print_error_messages_commands(&message, index);
                    self.execution_commands.push(None)
                }
            };
            index += 1;
        }
    }

    fn print_files_error(&mut self) {
        if self.infile_name == "here_doc" {
            if let Err(message) = &self.heredoc_content {
                self.print_error_message(&self.infile_name, message);
                self.infile_error = true;
            }
        } else {
            if let Err(message) = &self.infile_content {
                self.print_error_message(&self.infile_name, message);
                self.infile_error = true;
            }
        }
        if let Err(message) = &self.outfile_content {
            self.print_error_message(&self.outfile_name, message);
            self.outfile_error = true;
            self.exit_status = 1;
        }
    }

    fn print_error_message(&self, argument: &String, string: &String) {
        let trimmed_message: &str;

        if let Some(index_coincidence) = string.rfind(" (os error ") {
            trimmed_message = string[..index_coincidence].as_ref();
            eprintln!("rusted_pipex: {}: {}", argument, trimmed_message);
        }
    }
    fn handle_redirections(&self, command: &mut Command, index: usize) {
        if index == 1 && self.infile_error {
            command.stdin(Stdio::null());
        } else if index == 0 && !self.infile_error && !self.has_heredoc {
            if let Ok(file) = &self.infile_content {
                match file.try_clone() {
                    Ok(raw_file) => command.stdin(raw_file),
                    Err(_) => {
                        eprintln!(
                            "rusted_pipex: {}: Failed redirection of infile",
                            self.main_commands[index]
                        );
                        command.stdin(Stdio::null())
                    }
                };
            }
        } else {
            command.stdin(Stdio::piped());
        }
        if index == self.ammout_commands - 2 && self.outfile_error {
            command.stdout(Stdio::null());
        } else if index == self.ammout_commands - 1 && !self.outfile_error {
            if let Ok(file) = &self.outfile_content {
                match file.try_clone() {
                    Ok(raw_file) => command.stdout(raw_file),
                    Err(_) => {
                        eprintln!(
                            "rusted_pipex: {}: Failed redirection of outfile",
                            self.main_commands[index]
                        );
                        command.stdout(Stdio::null())
                    }
                };
            }
        } else {
            command.stdout(Stdio::piped());
        }
    }

    fn fill_stdin_heredoc(&self, child: &mut Child) {
        let raw_heredoc: &String;
        let write_status: Result<(), std::io::Error>;

        if let Ok(heredoc) = &self.heredoc_content {
            raw_heredoc = heredoc;
            if let Some(mut input) = child.stdin.take() {
                write_status = input.write_all(raw_heredoc.as_bytes());
                if let Err(error) = write_status {
                    self.format_error_message(&error, &self.infile_name);
                };
            };
        };
    }

    fn print_error_messages_commands(&mut self, error: &io::Error, index: usize) {
        let path_binary: PathBuf;

        path_binary = PathBuf::from(&self.main_commands[index]);
        if error.kind() == io::ErrorKind::NotFound && self.main_commands[index].find("/") == None {
            eprintln!(
                "rusted_pipex: {}: command not found",
                self.main_commands[index]
            );
        } else if error.kind() == io::ErrorKind::PermissionDenied && path_binary.is_dir() {
            eprintln!(
                "rusted_pipex: {}: is a directory",
                self.main_commands[index]
            );
        } else {
            self.format_error_message(error, &self.main_commands[index]);
        }
        if index == self.ammout_commands - 1
            && (error.kind() == io::ErrorKind::PermissionDenied || path_binary.is_dir())
        {
            self.exit_status = 126;
        } else if index == self.ammout_commands - 1 {
            self.exit_status = 127;
        }
    }

    fn format_error_message(&self, error: &io::Error, argument: &String) {
        let error_message: String;

        if error.kind() == io::ErrorKind::Other {
            eprintln!("rusted_pipex: {}: Undefined error", argument);
            return;
        }
        error_message = error.to_string();
        self.print_error_message(argument, &error_message);
        return;
    }

    pub fn iterate_commands(&mut self) {
        let mut index: usize;

        index = 0;
        while index < self.ammout_commands {
            if let Some(child_process) = &mut self.execution_commands[index] {
                if let Ok(exit_status) = child_process.wait() {
                    self.update_exit_status(&exit_status, index);
                } else {
                    eprintln!(
                        "rusted_pipex: {}: Error waiting to finish a child process",
                        self.main_commands[index]
                    );
                    if index == self.ammout_commands - 1 {
                        self.exit_status = 1;
                    }
                }
            }
            index += 1;
        }
    }

    fn update_exit_status(&mut self, exit_status: &ExitStatus, index: usize) {
        if index == self.ammout_commands - 1 {
            if let Some(exit_code) = exit_status.code() {
                self.exit_status = exit_code;
            } else {
                self.handle_signals(exit_status, &self.main_commands[index]);
            }
        }
    }

    fn handle_signals(&self, exit_status: &ExitStatus, command: &String) -> i32 {
        if let Some(signal_number) = exit_status.signal() {
            if signal_number == 13 {
                eprintln!("rusted_pipex: {}: Broken pipe", command);
            } else if signal_number == 11 {
                eprintln!("rusted_pipex: {}: Segmentation fault", command);
            } else if signal_number == 10 {
                eprintln!("rusted_pipex: {}: Bus Error", command);
            } else if signal_number == 9 {
                eprintln!("rusted_pipex: {}: Killed", command);
            } else if signal_number == 6 {
                eprintln!("rusted_pipex: {}: Abort", command);
            } else if signal_number == 3 {
                eprintln!("rusted_pipex: {}: Quit", command);
            } else {
                eprintln!("rusted_pipex: {}: Unknown signal", command);
            }
            return 128 + signal_number;
        }
        return 0;
    }

    pub fn get_exit_status(&self) -> u8 {
        return self.exit_status as u8;
    }
}
