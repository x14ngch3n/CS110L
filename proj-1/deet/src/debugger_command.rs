pub enum DebuggerCommand {
    Quit,
    Cont,
    Back,
    Break(String),
    Run(Vec<String>),
}

impl DebuggerCommand {
    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "c" | "cont" | "continue" => Some(DebuggerCommand::Cont),
            "bt" | "back" | "backtrace" => Some(DebuggerCommand::Back),
            "b" | "break" => {
                let breakpoint = tokens[1];
                Some(DebuggerCommand::Break(breakpoint.to_string()))
            }
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            // Default case:
            _ => None,
        }
    }
}
