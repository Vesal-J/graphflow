use std::fmt;

/// Top-level error enum representing any failure across the Graphflow framework.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphflowError {
    /// Errors originating from standard graph definition, compilation, or invocation.
    Graph(GraphError),
    /// Errors originating from the Pregel BSP execution engine.
    Pregel(PregelError),
    /// Errors originating from channel updates or type casting.
    Channel(ChannelError),
}

impl fmt::Display for GraphflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphflowError::Graph(err) => write!(f, "{err}"),
            GraphflowError::Pregel(err) => write!(f, "{err}"),
            GraphflowError::Channel(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for GraphflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GraphflowError::Graph(err) => Some(err),
            GraphflowError::Pregel(err) => Some(err),
            GraphflowError::Channel(err) => Some(err),
        }
    }
}

impl From<GraphError> for GraphflowError {
    fn from(err: GraphError) -> Self {
        GraphflowError::Graph(err)
    }
}

impl From<PregelError> for GraphflowError {
    fn from(err: PregelError) -> Self {
        GraphflowError::Pregel(err)
    }
}

impl From<ChannelError> for GraphflowError {
    fn from(err: ChannelError) -> Self {
        GraphflowError::Channel(err)
    }
}

/// Errors related to `Graph` compilation, structure validation, and runtime traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// Graph cannot be compiled because the entry point is not set.
    MissingEntryPoint,
    /// Graph cannot be compiled because the finish point is not set.
    MissingFinishPoint,
    /// The specified entry point node does not exist in the graph.
    EntryPointNotFound(String),
    /// The specified finish point node does not exist in the graph.
    FinishPointNotFound(String),
    /// An edge source node does not exist in the graph.
    EdgeSourceNotFound(String),
    /// An edge target node does not exist in the graph.
    EdgeTargetNotFound(String),
    /// A conditional edge source node does not exist in the graph.
    ConditionalEdgeSourceNotFound(String),
    /// A node referenced during invocation was not found in the graph.
    NodeNotFound(String),
    /// No outgoing edge was found for a non-finish node during graph traversal.
    MissingEdge(String),
    /// A node function failed during execution.
    ExecutionError(String),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::MissingEntryPoint => write!(f, "Entry point is not set"),
            GraphError::MissingFinishPoint => write!(f, "Finish point is not set"),
            GraphError::EntryPointNotFound(entry) => {
                write!(f, "Entry point '{entry}' does not exist")
            }
            GraphError::FinishPointNotFound(finish) => {
                write!(f, "Finish point '{finish}' does not exist")
            }
            GraphError::EdgeSourceNotFound(from) => {
                write!(f, "Edge source '{from}' does not exist")
            }
            GraphError::EdgeTargetNotFound(to) => {
                write!(f, "Edge target '{to}' does not exist")
            }
            GraphError::ConditionalEdgeSourceNotFound(from) => {
                write!(f, "Conditional edge source '{from}' does not exist")
            }
            GraphError::NodeNotFound(node) => {
                write!(f, "Node '{node}' does not exist")
            }
            GraphError::MissingEdge(node) => {
                write!(f, "No outgoing edge found from node '{node}'")
            }
            GraphError::ExecutionError(msg) => {
                write!(f, "Node execution failed: {msg}")
            }
        }
    }
}

impl std::error::Error for GraphError {}

impl From<fmt::Error> for GraphError {
    fn from(err: fmt::Error) -> Self {
        GraphError::ExecutionError(err.to_string())
    }
}

impl From<&str> for GraphError {
    fn from(msg: &str) -> Self {
        GraphError::ExecutionError(msg.to_string())
    }
}

impl From<String> for GraphError {
    fn from(msg: String) -> Self {
        GraphError::ExecutionError(msg)
    }
}

// Ergonomic comparison with string slices and Strings
#[allow(clippy::cmp_owned)]
impl PartialEq<&str> for GraphError {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

#[allow(clippy::cmp_owned)]
impl PartialEq<str> for GraphError {
    fn eq(&self, other: &str) -> bool {
        self.to_string() == other
    }
}

#[allow(clippy::cmp_owned)]
impl PartialEq<String> for GraphError {
    fn eq(&self, other: &String) -> bool {
        self.to_string() == *other
    }
}

#[allow(clippy::cmp_owned)]
impl PartialEq<GraphError> for &str {
    fn eq(&self, other: &GraphError) -> bool {
        *self == other.to_string()
    }
}

/// Errors related to channel operations in the Pregel BSP model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelError {
    /// A value supplied to the channel does not match the channel's expected type.
    TypeMismatch(String),
    /// Channel was accessed while empty when a value was strictly required.
    Empty(String),
    /// Custom channel update failure.
    Custom(String),
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelError::TypeMismatch(msg) => write!(f, "Type mismatch in channel: {msg}"),
            ChannelError::Empty(ch) => write!(f, "Channel '{ch}' is empty"),
            ChannelError::Custom(msg) => write!(f, "Channel error: {msg}"),
        }
    }
}

impl std::error::Error for ChannelError {}

impl From<&str> for ChannelError {
    fn from(msg: &str) -> Self {
        ChannelError::Custom(msg.to_string())
    }
}

impl From<String> for ChannelError {
    fn from(msg: String) -> Self {
        ChannelError::Custom(msg)
    }
}

/// Errors related to Pregel Bulk Synchronous Parallel execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PregelError {
    /// The specified channel does not exist in the Pregel application.
    ChannelNotFound(String),
    /// The specified node was not found.
    NodeNotFound(String),
    /// A required channel value was missing when read by a node or output selector.
    MissingChannel(String),
    /// The data type of a written value does not match the channel's declared type.
    TypeMismatch(String),
    /// The execution loop reached the maximum allowed superstep count without terminating.
    MaxStepsExceeded(usize),
    /// A worker thread or user-provided node function returned an execution error.
    ExecutionError(String),
    /// Pregel application configuration or compilation error.
    CompileError(String),
    /// Invocation inputs were empty or invalid.
    EmptyInput,
    /// An underlying channel operation failed.
    Channel(ChannelError),
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
            PregelError::Channel(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for PregelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PregelError::Channel(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ChannelError> for PregelError {
    fn from(err: ChannelError) -> Self {
        match err {
            ChannelError::TypeMismatch(msg) => PregelError::TypeMismatch(msg),
            other => PregelError::Channel(other),
        }
    }
}

impl From<&str> for PregelError {
    fn from(msg: &str) -> Self {
        PregelError::ExecutionError(msg.to_string())
    }
}

impl From<String> for PregelError {
    fn from(msg: String) -> Self {
        PregelError::ExecutionError(msg)
    }
}
