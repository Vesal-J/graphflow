use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum PregelError {
    ChannelNotFound(String),
    NodeNotFound(String),
    MissingChannel(String),
    TypeMismatch(String),
    MaxStepsExceeded(usize),
    ExecutionError(String),
    CompileError(String),
    EmptyInput,
}

impl fmt::Display for PregelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PregelError::ChannelNotFound(ch) => write!(f, "Channel not found: '{ch}'"),
            PregelError::NodeNotFound(node) => write!(f, "Node not found: '{node}'"),
            PregelError::MissingChannel(ch) => write!(f, "Missing value for channel: '{ch}'"),
            PregelError::TypeMismatch(msg) => write!(f, "Type mismatch: {msg}"),
            PregelError::MaxStepsExceeded(steps) => {
                write!(f, "Maximum supersteps exceeded ({steps})")
            }
            PregelError::ExecutionError(msg) => write!(f, "Execution error: {msg}"),
            PregelError::CompileError(msg) => write!(f, "Compile error: {msg}"),
            PregelError::EmptyInput => write!(f, "Input is empty or invalid"),
        }
    }
}

impl std::error::Error for PregelError {}
