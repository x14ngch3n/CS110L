use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::os::unix::prelude::CommandExt;
use std::process::{Child, Command};

use crate::dwarf_data::DwarfData;

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(std::mem::size_of::<usize>() as isize) as usize)
}

pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// hack a byte into original instruction
    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }

    /// hack 0xcc into original instruction, turn it into INT
    pub fn write_breakpoint(&mut self, addr: usize) -> Result<u8, nix::Error> {
        self.write_byte(addr, 0xcc)
    }

    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints: &Vec<usize>) -> Option<Inferior> {
        // implement me!
        let mut command = Command::new(target);
        command.args(args);
        unsafe {
            command.pre_exec(child_traceme);
        }
        let child = command.spawn().ok()?;
        let mut inferior = Inferior { child };
        // insert breakpoints before run
        for breakpoint in breakpoints.iter() {
            match inferior.write_breakpoint(*breakpoint) {
                Ok(_) => (),
                Err(_) => println!("Fail to insert breakpoint at {:#x}", breakpoint),
            }
        }
        Some(inferior)
    }

    /// Wakes up the inferior and waits until it stops or terminates
    pub fn continue_run(&mut self, signal: Option<signal::Signal>) -> Result<Status, nix::Error> {
        ptrace::cont(self.pid(), signal)?;
        self.wait(None)
    }

    /// Kill the existing child process and reap it
    pub fn kill(&mut self) -> Result<(), nix::Error> {
        self.child.kill().expect("Process is not running");
        self.wait(None).expect("Fail to reap the killed process");
        println!("Killing running inferior (pid {})", self.pid());
        Ok(())
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Print backtrace of current status till main
    pub fn print_backtrace(&mut self, debug_data: &DwarfData) -> Result<(), nix::Error> {
        let regs = ptrace::getregs(self.pid()).unwrap();
        let mut instruction_ptr = regs.rip as usize;
        let mut stackbase_ptr = regs.rbp as usize;
        let mut backtraces = Vec::new();
        loop {
            let function = debug_data.get_function_from_addr(instruction_ptr).unwrap();
            let line = debug_data.get_line_from_addr(instruction_ptr).unwrap();
            backtraces.push(format!("{} ({})", function, line));
            if function == String::from("main") {
                break;
            }
            instruction_ptr = ptrace::read(self.pid(), (stackbase_ptr + 8) as ptrace::AddressType)
                .unwrap() as usize;
            stackbase_ptr =
                ptrace::read(self.pid(), stackbase_ptr as ptrace::AddressType).unwrap() as usize;
        }
        for backtrace in backtraces.iter().rev() {
            println!("{}", backtrace);
        }
        Ok(())
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }
}
