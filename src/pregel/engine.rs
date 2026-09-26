use std::any::Any;
use std::collections::HashMap;

use crate::pregel::channel::Channel;
use crate::pregel::errors::PregelError;
use crate::pregel::node::{ChannelValues, ChannelWriteEntry, PregelNode};

/// Pregel Bulk Synchronous Parallel execution engine.
pub struct Pregel {
    pub nodes: HashMap<String, PregelNode>,
    pub channels: HashMap<String, Box<dyn Channel>>,
    pub input_channels: Vec<String>,
    pub output_channels: Vec<String>,
    pub max_steps: usize,
}

impl Default for Pregel {
    fn default() -> Self {
        Self::new()
    }
}

impl Pregel {
    /// Creates a new empty `Pregel` application.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            channels: HashMap::new(),
            input_channels: Vec::new(),
            output_channels: Vec::new(),
            max_steps: 25,
        }
    }

    /// Registers a channel with the given name.
    pub fn add_channel<C: Channel + 'static>(
        &mut self,
        name: impl Into<String>,
        channel: C,
    ) -> &mut Self {
        self.channels.insert(name.into(), Box::new(channel));
        self
    }

    /// Registers a pregel node / actor.
    pub fn add_node(&mut self, node: impl Into<PregelNode>) -> &mut Self {
        let node = node.into();
        self.nodes.insert(node.name.clone(), node);
        self
    }

    /// Configures the designated input channels.
    pub fn set_input_channels(&mut self, channels: Vec<impl Into<String>>) -> &mut Self {
        self.input_channels = channels.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Configures the channels to return in the final output.
    /// If left empty, all channels that hold values will be returned.
    pub fn set_output_channels(&mut self, channels: Vec<impl Into<String>>) -> &mut Self {
        self.output_channels = channels.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Sets the maximum number of supersteps allowed before returning `PregelError::MaxStepsExceeded`.
    pub fn set_max_steps(&mut self, max: usize) -> &mut Self {
        self.max_steps = max;
        self
    }

    /// Executes the Pregel application with the provided `ChannelValues`.
    pub fn invoke_values(&self, inputs: ChannelValues) -> Result<ChannelValues, PregelError> {
        if inputs.is_empty() {
            return Err(PregelError::EmptyInput);
        }

        // Validate that all input channels exist
        for in_ch in inputs.values.keys() {
            if !self.channels.contains_key(in_ch) {
                return Err(PregelError::ChannelNotFound(in_ch.clone()));
            }
        }

        // Clone channels for isolated, thread-safe execution
        let mut channels: HashMap<String, Box<dyn Channel>> = HashMap::new();
        for (name, ch) in &self.channels {
            channels.insert(name.clone(), ch.clone_channel());
        }

        // Apply initial inputs to channels
        for (in_ch, val_boxed) in inputs.values {
            if let Some(ch) = channels.get_mut(&in_ch) {
                ch.update(vec![val_boxed])
                    .map_err(PregelError::TypeMismatch)?;
            }
        }

        let mut step = 0;
        while step < self.max_steps {
            // ---------------------------------------------------------------
            // Phase 1: PLAN
            // Determine which nodes are triggered by channels updated in the
            // previous step (or by initial inputs in step 0).
            // ---------------------------------------------------------------
            let mut active_nodes = Vec::new();
            for (node_name, node) in &self.nodes {
                let is_triggered = node.triggers.iter().any(|trig| {
                    channels
                        .get(trig)
                        .map(|c| c.is_updated())
                        .unwrap_or(false)
                });
                if is_triggered {
                    active_nodes.push(node_name.clone());
                }
            }

            // Quiescence: no active nodes left, execution finishes
            if active_nodes.is_empty() {
                break;
            }

            // Deterministic sort of active nodes
            active_nodes.sort();

            // ---------------------------------------------------------------
            // Phase 2: EXECUTION
            // Run all selected nodes in parallel without mutex contention.
            // Nodes read an immutable snapshot of channels; channel updates
            // made during this phase are invisible until Phase 3.
            // ---------------------------------------------------------------
            let mut snapshot = ChannelValues::new();
            for (ch_name, ch) in &channels {
                if let Some(boxed) = ch.get_boxed() {
                    snapshot.insert_boxed(ch_name.clone(), boxed);
                }
            }

            let all_writes: Vec<ChannelWriteEntry> = if active_nodes.len() == 1 {
                let node_name = &active_nodes[0];
                let node = self
                    .nodes
                    .get(node_name)
                    .ok_or_else(|| PregelError::NodeNotFound(node_name.clone()))?;
                (node.executor)(&snapshot)?
            } else {
                let snapshot_ref = &snapshot;
                let nodes_ref = &self.nodes;

                let results: Result<Vec<Vec<ChannelWriteEntry>>, PregelError> =
                    std::thread::scope(|s| {
                        let mut handles = Vec::new();
                        for node_name in &active_nodes {
                            let node = nodes_ref
                                .get(node_name)
                                .ok_or_else(|| PregelError::NodeNotFound(node_name.clone()))?;
                            let handle =
                                s.spawn(move || -> Result<Vec<ChannelWriteEntry>, PregelError> {
                                    (node.executor)(snapshot_ref)
                                });
                            handles.push(handle);
                        }

                        let mut collected = Vec::new();
                        for handle in handles {
                            let res = handle.join().map_err(|_| {
                                PregelError::ExecutionError(
                                    "A worker thread panicked during node execution".to_string(),
                                )
                            })??;
                            collected.push(res);
                        }
                        Ok(collected)
                    });

                let mut combined = Vec::new();
                for batch in results? {
                    combined.extend(batch);
                }
                combined
            };

            // If active nodes produced no writes, execution has quiesced/finished
            if all_writes.is_empty() {
                break;
            }

            // ---------------------------------------------------------------
            // Phase 3: UPDATE
            // Synchronization barrier: aggregate all writes and update channels.
            // ---------------------------------------------------------------
            let mut grouped_writes: HashMap<String, Vec<Box<dyn Any + Send + Sync>>> =
                HashMap::new();
            for entry in all_writes {
                if !channels.contains_key(&entry.channel) {
                    return Err(PregelError::ChannelNotFound(entry.channel));
                }
                grouped_writes
                    .entry(entry.channel)
                    .or_default()
                    .push(entry.value);
            }

            // Step reset (clears ephemeral values, resets is_updated flags)
            for ch in channels.values_mut() {
                ch.step_reset();
            }

            // Apply grouped writes to channels
            for (target, writes) in grouped_writes {
                if let Some(ch) = channels.get_mut(&target) {
                    ch.update(writes).map_err(PregelError::TypeMismatch)?;
                }
            }

            step += 1;
        }

        if step >= self.max_steps {
            return Err(PregelError::MaxStepsExceeded(self.max_steps));
        }

        // ---------------------------------------------------------------
        // Produce final output
        // ---------------------------------------------------------------
        let mut output = ChannelValues::new();
        if self.output_channels.is_empty() {
            for (name, ch) in &channels {
                if let Some(boxed) = ch.get_boxed() {
                    output.insert_boxed(name.clone(), boxed);
                }
            }
        } else {
            for name in &self.output_channels {
                if let Some(ch) = channels.get(name) {
                    if let Some(boxed) = ch.get_boxed() {
                        output.insert_boxed(name.clone(), boxed);
                    }
                }
            }
        }

        Ok(output)
    }

    /// Convenience invocation taking and returning standard HashMaps with Box<dyn Any>.
    pub fn invoke(
        &self,
        inputs: HashMap<String, Box<dyn Any + Send + Sync>>,
    ) -> Result<HashMap<String, Box<dyn Any + Send + Sync>>, PregelError> {
        let mut channel_values = ChannelValues::new();
        for (k, v) in inputs {
            channel_values.insert_boxed(k, v);
        }
        let res = self.invoke_values(channel_values)?;
        Ok(res.values)
    }

    /// Single input to single output channel convenience invocation.
    pub fn invoke_single<In: Send + Sync + 'static, Out: Clone + 'static>(
        &self,
        input_channel: &str,
        input: In,
        output_channel: &str,
    ) -> Result<Out, PregelError> {
        let mut inputs = ChannelValues::new();
        inputs.insert(input_channel, input);
        let outputs = self.invoke_values(inputs)?;
        outputs
            .get::<Out>(output_channel)
            .ok_or_else(|| PregelError::MissingChannel(output_channel.to_string()))
    }
}
