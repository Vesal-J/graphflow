mod compiled_graph;
pub mod errors;
mod graph;
pub mod logging;
pub mod pregel;

pub use compiled_graph::CompiledGraph;
pub use errors::{ChannelError, GraphError, GraphflowError, PregelError};
pub use graph::{BranchFunction, Graph, NodeFunction};
pub use logging::{init as init_logger, init_with_level as init_logger_with_level};
pub use pregel::{
    BinaryOperatorAggregate, Channel, ChannelValues, ChannelWriteEntry, EphemeralValue, LastValue,
    NodeBuilder, Pregel, PregelNode, Topic,
};

#[cfg(test)]
mod tests;
