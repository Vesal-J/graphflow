use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use crate::pregel::errors::PregelError;

/// A snapshot container of channel values passed to nodes during execution.
#[derive(Default)]
pub struct ChannelValues {
    pub values: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl fmt::Debug for ChannelValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_map = f.debug_map();
        for k in self.values.keys() {
            debug_map.entry(k, &"<channel_value>");
        }
        debug_map.finish()
    }
}

impl ChannelValues {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn insert<V: Send + Sync + 'static>(
        &mut self,
        channel: impl Into<String>,
        value: V,
    ) -> &mut Self {
        self.values.insert(channel.into(), Box::new(value));
        self
    }

    pub fn insert_boxed(
        &mut self,
        channel: impl Into<String>,
        boxed: Box<dyn Any + Send + Sync>,
    ) -> &mut Self {
        self.values.insert(channel.into(), boxed);
        self
    }

    pub fn get<V: Clone + 'static>(&self, channel: &str) -> Option<V> {
        self.values
            .get(channel)
            .and_then(|b| b.downcast_ref::<V>().cloned())
    }

    pub fn get_ref<V: 'static>(&self, channel: &str) -> Option<&V> {
        self.values
            .get(channel)
            .and_then(|b| b.downcast_ref::<V>())
    }

    pub fn contains_key(&self, channel: &str) -> bool {
        self.values.contains_key(channel)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Represents a single write directed to a channel.
pub struct ChannelWriteEntry {
    pub channel: String,
    pub value: Box<dyn Any + Send + Sync>,
}

impl ChannelWriteEntry {
    pub fn new<V: Send + Sync + 'static>(channel: impl Into<String>, value: V) -> Self {
        Self {
            channel: channel.into(),
            value: Box::new(value),
        }
    }

    pub fn boxed(channel: impl Into<String>, boxed: Box<dyn Any + Send + Sync>) -> Self {
        Self {
            channel: channel.into(),
            value: boxed,
        }
    }
}

/// An Actor in the Pregel Bulk Synchronous Parallel execution model.
pub struct PregelNode {
    pub name: String,
    pub triggers: Vec<String>,
    pub read_channels: Vec<String>,
    pub write_targets: Vec<String>,
    #[allow(clippy::type_complexity)]
    pub executor:
        Box<dyn Fn(&ChannelValues) -> Result<Vec<ChannelWriteEntry>, PregelError> + Send + Sync>,
}

type NodeRunner = Box<
    dyn Fn(&ChannelValues, &[String]) -> Result<Vec<ChannelWriteEntry>, PregelError> + Send + Sync,
>;

/// Fluent builder for constructing `PregelNode`s with LangGraph-compatible ergonomics.
pub struct NodeBuilder {
    pub name: String,
    pub triggers: Vec<String>,
    pub read_channels: Vec<String>,
    pub write_targets: Vec<String>,
    pub skip_none: bool,
    runner: Option<NodeRunner>,
}

impl NodeBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            triggers: Vec::new(),
            read_channels: Vec::new(),
            write_targets: Vec::new(),
            skip_none: false,
            runner: None,
        }
    }

    /// Subscribes to and reads from a single channel. Updates to this channel trigger the node.
    pub fn subscribe_only(mut self, channel: impl Into<String>) -> Self {
        let ch = channel.into();
        self.triggers = vec![ch.clone()];
        self.read_channels = vec![ch];
        self
    }

    /// Subscribes to multiple channels. An update to ANY of these channels triggers the node.
    pub fn subscribe_to(mut self, channels: Vec<impl Into<String>>) -> Self {
        let chs: Vec<String> = channels.into_iter().map(|s| s.into()).collect();
        self.triggers = chs.clone();
        self.read_channels = chs;
        self
    }

    /// Adds a target channel to which this node writes its result.
    pub fn write_to(mut self, channel: impl Into<String>) -> Self {
        self.write_targets.push(channel.into());
        self
    }

    /// Adds multiple target channels to which this node writes its result.
    pub fn write_to_many(mut self, channels: Vec<impl Into<String>>) -> Self {
        for ch in channels {
            self.write_targets.push(ch.into());
        }
        self
    }

    /// If set to true, `None` outputs will not be written to channels.
    pub fn skip_none(mut self, skip: bool) -> Self {
        self.skip_none = skip;
        self
    }

    /// Connects a pure transformation function `Fn(In) -> Out`.
    pub fn do_pure<In, Out, F>(mut self, func: F) -> Self
    where
        In: Clone + 'static,
        Out: Clone + Send + Sync + 'static,
        F: Fn(In) -> Out + Send + Sync + 'static,
    {
        let read_ch = self
            .read_channels
            .first()
            .cloned()
            .unwrap_or_else(|| self.name.clone());

        self.runner = Some(Box::new(
            move |values: &ChannelValues,
                  write_targets: &[String]|
                  -> Result<Vec<ChannelWriteEntry>, PregelError> {
                let input = values
                    .get::<In>(&read_ch)
                    .ok_or_else(|| PregelError::MissingChannel(read_ch.clone()))?;
                let output = func(input);
                let mut entries = Vec::with_capacity(write_targets.len());
                for target in write_targets {
                    entries.push(ChannelWriteEntry::new(target.clone(), output.clone()));
                }
                Ok(entries)
            },
        ));
        self
    }

    /// Connects a fallible function `Fn(In) -> Result<Out, E>`.
    pub fn do_fn<In, Out, F, E>(mut self, func: F) -> Self
    where
        In: Clone + 'static,
        Out: Clone + Send + Sync + 'static,
        F: Fn(In) -> Result<Out, E> + Send + Sync + 'static,
        E: std::fmt::Display,
    {
        let read_ch = self
            .read_channels
            .first()
            .cloned()
            .unwrap_or_else(|| self.name.clone());

        self.runner = Some(Box::new(
            move |values: &ChannelValues,
                  write_targets: &[String]|
                  -> Result<Vec<ChannelWriteEntry>, PregelError> {
                let input = values
                    .get::<In>(&read_ch)
                    .ok_or_else(|| PregelError::MissingChannel(read_ch.clone()))?;
                let output =
                    func(input).map_err(|e| PregelError::ExecutionError(e.to_string()))?;
                let mut entries = Vec::with_capacity(write_targets.len());
                for target in write_targets {
                    entries.push(ChannelWriteEntry::new(target.clone(), output.clone()));
                }
                Ok(entries)
            },
        ));
        self
    }

    /// Connects a pure function returning an `Option<Out>`.
    /// If `None` is returned, no write entry is produced (satisfying `skip_none` behavior for cycles).
    pub fn do_pure_option<In, Out, F>(mut self, func: F) -> Self
    where
        In: Clone + 'static,
        Out: Clone + Send + Sync + 'static,
        F: Fn(In) -> Option<Out> + Send + Sync + 'static,
    {
        let read_ch = self
            .read_channels
            .first()
            .cloned()
            .unwrap_or_else(|| self.name.clone());

        self.runner = Some(Box::new(
            move |values: &ChannelValues,
                  write_targets: &[String]|
                  -> Result<Vec<ChannelWriteEntry>, PregelError> {
                let input = values
                    .get::<In>(&read_ch)
                    .ok_or_else(|| PregelError::MissingChannel(read_ch.clone()))?;
                let output = func(input);
                let mut entries = Vec::new();
                if let Some(val) = output {
                    for target in write_targets {
                        entries.push(ChannelWriteEntry::new(target.clone(), val.clone()));
                    }
                }
                Ok(entries)
            },
        ));
        self
    }

    /// Connects a fallible function returning an `Option<Out>`.
    pub fn do_option_fn<In, Out, F, E>(mut self, func: F) -> Self
    where
        In: Clone + 'static,
        Out: Clone + Send + Sync + 'static,
        F: Fn(In) -> Result<Option<Out>, E> + Send + Sync + 'static,
        E: std::fmt::Display,
    {
        let read_ch = self
            .read_channels
            .first()
            .cloned()
            .unwrap_or_else(|| self.name.clone());

        self.runner = Some(Box::new(
            move |values: &ChannelValues,
                  write_targets: &[String]|
                  -> Result<Vec<ChannelWriteEntry>, PregelError> {
                let input = values
                    .get::<In>(&read_ch)
                    .ok_or_else(|| PregelError::MissingChannel(read_ch.clone()))?;
                let output =
                    func(input).map_err(|e| PregelError::ExecutionError(e.to_string()))?;
                let mut entries = Vec::new();
                if let Some(val) = output {
                    for target in write_targets {
                        entries.push(ChannelWriteEntry::new(target.clone(), val.clone()));
                    }
                }
                Ok(entries)
            },
        ));
        self
    }

    /// Connects a custom function that has access to all channel values and produces arbitrary writes.
    pub fn do_values_fn<F, E>(mut self, func: F) -> Self
    where
        F: Fn(&ChannelValues) -> Result<Vec<ChannelWriteEntry>, E> + Send + Sync + 'static,
        E: std::fmt::Display,
    {
        self.runner = Some(Box::new(
            move |values: &ChannelValues,
                  _write_targets: &[String]|
                  -> Result<Vec<ChannelWriteEntry>, PregelError> {
                func(values).map_err(|e| PregelError::ExecutionError(e.to_string()))
            },
        ));
        self
    }

    /// Connects a pure function that has access to all channel values and produces arbitrary writes without errors.
    pub fn do_values_pure<F>(mut self, func: F) -> Self
    where
        F: Fn(&ChannelValues) -> Vec<ChannelWriteEntry> + Send + Sync + 'static,
    {
        self.runner = Some(Box::new(
            move |values: &ChannelValues,
                  _write_targets: &[String]|
                  -> Result<Vec<ChannelWriteEntry>, PregelError> {
                Ok(func(values))
            },
        ));
        self
    }

    /// Compiles the `NodeBuilder` into an executable `PregelNode`.
    pub fn build(self) -> PregelNode {
        let write_targets = self.write_targets.clone();
        let runner = self.runner.unwrap_or_else(|| {
            Box::new(|_, _| Ok(Vec::new()))
        });

        let executor = Box::new(
            move |values: &ChannelValues| -> Result<Vec<ChannelWriteEntry>, PregelError> {
                runner(values, &write_targets)
            },
        );

        PregelNode {
            name: self.name,
            triggers: self.triggers,
            read_channels: self.read_channels,
            write_targets: self.write_targets,
            executor,
        }
    }
}

impl From<NodeBuilder> for PregelNode {
    fn from(builder: NodeBuilder) -> Self {
        builder.build()
    }
}
