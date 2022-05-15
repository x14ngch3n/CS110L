use crate::debugger_command::DebuggerCommand;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct BreakPoint {
    id: usize,
    addr: usize,
    orig_byte: u8,
}

impl BreakPoint {
    fn new(id: usize, addr: usize) -> Self {
        BreakPoint {
            id,
            addr,
            orig_byte: 0,
        }
    }

    pub fn addr(&self) -> usize {
        self.addr
    }

    pub fn set_byte(&mut self, orig_byte: u8) {
        self.orig_byte = orig_byte
    }
}

impl std::fmt::Display for BreakPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {} ", self.id).unwrap();
        write!(f, "ADDR: {:#x} ", self.addr).unwrap();
        write!(f, "ORIN_BYTE: {} ", self.orig_byte)
    }
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: HashMap<usize, BreakPoint>,
}

fn parse_address(address: &str) -> Option<usize> {
    let address_without_0x = if address.to_lowercase().starts_with("0x") {
        &address[2..]
    } else {
        &address[..]
    };
    usize::from_str_radix(address_without_0x, 16).ok()
}

enum BreakPointType<'a> {
    Raw(&'a str),
    Line(usize),
    Func(&'a str),
}

fn get_breakpoint_type(breakpoint: &str) -> BreakPointType {
    if breakpoint.starts_with('*') {
        return BreakPointType::Raw(&breakpoint[1..]);
    }
    match usize::from_str_radix(breakpoint, 10) {
        Ok(line) => BreakPointType::Line(line),
        Err(_) => BreakPointType::Func(breakpoint),
    }
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill().unwrap();
                    }
                    if let Some(inferior) =
                        Inferior::new(&self.target, &args, &mut self.breakpoints)
                    {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        match self.inferior.as_mut().unwrap().continue_run(None).unwrap() {
                            Status::Exited(exit_code) => {
                                println!("Child exited (status {})", exit_code);
                                self.inferior = None;
                            }
                            Status::Signaled(singal) => {
                                println!("Child exited with {}", singal);
                                self.inferior = None;
                            }
                            Status::Stopped(signal, rip) => {
                                println!("Child stopped with {} at address {:#x}", signal, rip);
                                let function = self.debug_data.get_function_from_addr(rip);
                                let line = self.debug_data.get_line_from_addr(rip);
                                match (function, line) {
                                    (Some(function), Some(line)) => {
                                        println!("Stopped at {} ({})", function, line)
                                    }
                                    (_, _) => {
                                        println!("Fail to resolve stopping function and line")
                                    }
                                }
                            }
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Cont => {
                    if self.inferior.is_none() {
                        println!("The process is not being run");
                        continue;
                    }
                    // check if stop in breakpoint
                    let rip = self.inferior.as_ref().unwrap().get_previous_ins().unwrap();
                    if self.breakpoints.contains_key(&rip) {
                        println!(
                            "Previously Stopped at breakpoint: {}\n",
                            self.breakpoints.get(&rip).unwrap()
                        );
                        if !self
                            .inferior
                            .as_mut()
                            .unwrap()
                            .step_breakpoint(rip, self.breakpoints.get(&rip).unwrap().orig_byte)
                        {
                            println!("Failed to step by the breakpoint");
                            continue;
                        }
                    }
                    match self.inferior.as_mut().unwrap().continue_run(None).unwrap() {
                        Status::Exited(exit_code) => {
                            println!("Child exited (status {})", exit_code);
                            self.inferior = None;
                        }
                        Status::Signaled(singal) => {
                            println!("Child exited with {}", singal);
                            self.inferior = None;
                        }
                        Status::Stopped(signal, rip) => {
                            println!("Child stopped with {} at address {:#x}", signal, rip);
                            let function = self.debug_data.get_function_from_addr(rip);
                            let line = self.debug_data.get_line_from_addr(rip);
                            match (function, line) {
                                (Some(function), Some(line)) => {
                                    println!("Stopped at {} ({})", function, line)
                                }
                                (_, _) => {
                                    println!("Fail to resolve stopping function and line")
                                }
                            }
                        }
                    }
                }
                DebuggerCommand::Back => {
                    if self.inferior.is_none() {
                        println!("The process is not being run");
                        continue;
                    }
                    self.inferior
                        .as_mut()
                        .unwrap()
                        .print_backtrace(&self.debug_data)
                        .unwrap();
                }
                DebuggerCommand::Break(breakpoint) => {
                    let breakpoint = match get_breakpoint_type(&breakpoint) {
                        BreakPointType::Raw(address) => parse_address(address).unwrap(),
                        // unable to get lines info in dwarf file, don't know why
                        BreakPointType::Line(line) => {
                            match self.debug_data.get_addr_for_line(None, line) {
                                Some(addr) => addr,
                                None => {
                                    println!("Failed to find the address of line {}", line);
                                    continue;
                                }
                            }
                        }
                        BreakPointType::Func(func) => {
                            match self.debug_data.get_addr_for_function(None, func) {
                                Some(addr) => addr,
                                None => {
                                    println!("Failed to find the address of function {}", func);
                                    continue;
                                }
                            }
                        }
                    };
                    if !self.breakpoints.contains_key(&breakpoint) {
                        // add breakpoint to global Hashmap, without knowing the orig_byte
                        self.breakpoints.insert(
                            breakpoint,
                            BreakPoint::new(self.breakpoints.len() + 1, breakpoint),
                        );
                        // add breakpoint when process is stopped
                        if self.inferior.is_some() {
                            match self.inferior.as_mut().unwrap().write_breakpoint(breakpoint) {
                                Ok(orig_byte) => self
                                    .breakpoints
                                    .get_mut(&breakpoint)
                                    .unwrap()
                                    .set_byte(orig_byte),
                                Err(_) => {
                                    println!("Fail to insert breakpoint at {:#x}", breakpoint);
                                    continue;
                                }
                            }
                        }
                    }
                    println!(
                        "Set breakpoint {} at {}",
                        self.breakpoints.get(&breakpoint).as_ref().unwrap().id,
                        breakpoint
                    )
                }
                DebuggerCommand::Quit => {
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill().unwrap();
                    }
                    return;
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
