mod compiled_graph;
mod graph;
pub mod pregel;

pub use compiled_graph::CompiledGraph;
pub use graph::Graph;
pub use pregel::{
    BinaryOperatorAggregate, Channel, ChannelValues, ChannelWriteEntry, EphemeralValue, LastValue,
    NodeBuilder, Pregel, PregelError, PregelNode, Topic,
};

#[cfg(test)]
mod tests;

