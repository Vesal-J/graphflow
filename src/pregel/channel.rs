use std::any::Any;
use std::sync::Arc;

use crate::errors::ChannelError;

pub trait Channel: Send + Sync {
    /// Returns true if this channel was updated in the most recent superstep.
    fn is_updated(&self) -> bool;

    /// Called at the transition between supersteps.
    /// Resets transient state (e.g. clears ephemeral values and resets the `is_updated` flag).
    fn step_reset(&mut self);

    /// Applies a batch of values written to this channel during the Execution phase.
    fn update(&mut self, values: Vec<Box<dyn Any + Send + Sync>>) -> Result<(), ChannelError>;

    /// Returns a boxed clone of the current channel value, if present.
    fn get_boxed(&self) -> Option<Box<dyn Any + Send + Sync>>;

    /// Clones the channel into a new `Box<dyn Channel>`.
    fn clone_channel(&self) -> Box<dyn Channel>;
}

// ---------------------------------------------------------------------------
// LastValue Channel
// ---------------------------------------------------------------------------

/// Stores the last value sent to the channel.
/// The value persists across supersteps until a new write overwrites it.
#[derive(Clone, Debug)]
pub struct LastValue<V: Clone + Send + Sync + 'static> {
    pub value: Option<V>,
    pub updated: bool,
}

impl<V: Clone + Send + Sync + 'static> LastValue<V> {
    pub fn new() -> Self {
        Self {
            value: None,
            updated: false,
        }
    }

    pub fn with_initial(val: V) -> Self {
        Self {
            value: Some(val),
            updated: false,
        }
    }
}

impl<V: Clone + Send + Sync + 'static> Default for LastValue<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone + Send + Sync + 'static> Channel for LastValue<V> {
    fn is_updated(&self) -> bool {
        self.updated
    }

    fn step_reset(&mut self) {
        self.updated = false;
    }

    fn update(&mut self, values: Vec<Box<dyn Any + Send + Sync>>) -> Result<(), ChannelError> {
        if values.is_empty() {
            return Ok(());
        }
        if let Some(last) = values.into_iter().last() {
            let val = last
                .downcast::<V>()
                .map_err(|_| ChannelError::TypeMismatch("Type mismatch in LastValue channel".to_string()))?;
            self.value = Some(*val);
            self.updated = true;
            log::trace!("LastValue channel updated with new value");
        }
        Ok(())
    }

    fn get_boxed(&self) -> Option<Box<dyn Any + Send + Sync>> {
        self.value
            .as_ref()
            .map(|v| Box::new(v.clone()) as Box<dyn Any + Send + Sync>)
    }

    fn clone_channel(&self) -> Box<dyn Channel> {
        Box::new(self.clone())
    }
}

// ---------------------------------------------------------------------------
// EphemeralValue Channel
// ---------------------------------------------------------------------------

/// Stores a value for exactly ONE superstep.
/// At the beginning of the next superstep update, if no new value is written,
/// the value is cleared (resets to `None`).
#[derive(Clone, Debug)]
pub struct EphemeralValue<V: Clone + Send + Sync + 'static> {
    pub value: Option<V>,
    pub updated: bool,
}

impl<V: Clone + Send + Sync + 'static> EphemeralValue<V> {
    pub fn new() -> Self {
        Self {
            value: None,
            updated: false,
        }
    }
}

impl<V: Clone + Send + Sync + 'static> Default for EphemeralValue<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Clone + Send + Sync + 'static> Channel for EphemeralValue<V> {
    fn is_updated(&self) -> bool {
        self.updated
    }

    fn step_reset(&mut self) {
        self.value = None;
        self.updated = false;
        log::trace!("EphemeralValue channel reset (cleared)");
    }

    fn update(&mut self, values: Vec<Box<dyn Any + Send + Sync>>) -> Result<(), ChannelError> {
        if values.is_empty() {
            return Ok(());
        }
        if let Some(last) = values.into_iter().last() {
            let val = last
                .downcast::<V>()
                .map_err(|_| ChannelError::TypeMismatch("Type mismatch in EphemeralValue channel".to_string()))?;
            self.value = Some(*val);
            self.updated = true;
            log::trace!("EphemeralValue channel updated with new value");
        }
        Ok(())
    }

    fn get_boxed(&self) -> Option<Box<dyn Any + Send + Sync>> {
        self.value
            .as_ref()
            .map(|v| Box::new(v.clone()) as Box<dyn Any + Send + Sync>)
    }

    fn clone_channel(&self) -> Box<dyn Channel> {
        Box::new(self.clone())
    }
}

// ---------------------------------------------------------------------------
// Topic Channel
// ---------------------------------------------------------------------------

/// PubSub Topic channel. Accumulates multiple values sent to it.
/// If `accumulate` is true, values persist and accumulate across all supersteps.
/// If `accumulate` is false, values are cleared after each superstep.
#[derive(Clone, Debug)]
pub struct Topic<V: Clone + Send + Sync + PartialEq + 'static> {
    pub values: Vec<V>,
    pub updated: bool,
    pub accumulate: bool,
    pub dedup: bool,
}

impl<V: Clone + Send + Sync + PartialEq + 'static> Topic<V> {
    pub fn new(accumulate: bool) -> Self {
        Self {
            values: Vec::new(),
            updated: false,
            accumulate,
            dedup: false,
        }
    }

    pub fn with_dedup(accumulate: bool, dedup: bool) -> Self {
        Self {
            values: Vec::new(),
            updated: false,
            accumulate,
            dedup,
        }
    }
}

impl<V: Clone + Send + Sync + PartialEq + 'static> Channel for Topic<V> {
    fn is_updated(&self) -> bool {
        self.updated
    }

    fn step_reset(&mut self) {
        if !self.accumulate {
            self.values.clear();
            log::trace!("Topic channel non-accumulating reset (cleared values)");
        }
        self.updated = false;
    }

    fn update(&mut self, values: Vec<Box<dyn Any + Send + Sync>>) -> Result<(), ChannelError> {
        if values.is_empty() {
            return Ok(());
        }
        let mut added = false;
        for boxed in values {
            let val = *boxed
                .downcast::<V>()
                .map_err(|_| ChannelError::TypeMismatch("Type mismatch in Topic channel".to_string()))?;
            if self.dedup && self.values.contains(&val) {
                continue;
            }
            self.values.push(val);
            added = true;
        }
        if added {
            self.updated = true;
            log::trace!("Topic channel updated (count: {})", self.values.len());
        }
        Ok(())
    }

    fn get_boxed(&self) -> Option<Box<dyn Any + Send + Sync>> {
        Some(Box::new(self.values.clone()) as Box<dyn Any + Send + Sync>)
    }

    fn clone_channel(&self) -> Box<dyn Channel> {
        Box::new(self.clone())
    }
}

// ---------------------------------------------------------------------------
// BinaryOperatorAggregate Channel
// ---------------------------------------------------------------------------

/// Stores a persistent value updated by applying a binary operator / reducer function
/// to the current value and each update sent to the channel.
#[derive(Clone)]
pub struct BinaryOperatorAggregate<V: Clone + Send + Sync + 'static> {
    pub value: Option<V>,
    pub updated: bool,
    #[allow(clippy::type_complexity)]
    pub reducer: Arc<dyn Fn(Option<V>, V) -> V + Send + Sync>,
}

impl<V: Clone + Send + Sync + 'static> BinaryOperatorAggregate<V> {
    pub fn new<F>(reducer: F) -> Self
    where
        F: Fn(Option<V>, V) -> V + Send + Sync + 'static,
    {
        Self {
            value: None,
            updated: false,
            reducer: Arc::new(reducer),
        }
    }

    pub fn with_initial<F>(initial: V, reducer: F) -> Self
    where
        F: Fn(Option<V>, V) -> V + Send + Sync + 'static,
    {
        Self {
            value: Some(initial),
            updated: false,
            reducer: Arc::new(reducer),
        }
    }
}

impl<V: Clone + Send + Sync + 'static> Channel for BinaryOperatorAggregate<V> {
    fn is_updated(&self) -> bool {
        self.updated
    }

    fn step_reset(&mut self) {
        self.updated = false;
    }

    fn update(&mut self, values: Vec<Box<dyn Any + Send + Sync>>) -> Result<(), ChannelError> {
        if values.is_empty() {
            return Ok(());
        }
        for boxed in values {
            let val = *boxed
                .downcast::<V>()
                .map_err(|_| ChannelError::TypeMismatch("Type mismatch in BinaryOperatorAggregate channel".to_string()))?;
            let current = self.value.take();
            let new_val = (self.reducer)(current, val);
            self.value = Some(new_val);
            self.updated = true;
        }
        log::trace!("BinaryOperatorAggregate channel updated via reducer");
        Ok(())
    }

    fn get_boxed(&self) -> Option<Box<dyn Any + Send + Sync>> {
        self.value
            .as_ref()
            .map(|v| Box::new(v.clone()) as Box<dyn Any + Send + Sync>)
    }

    fn clone_channel(&self) -> Box<dyn Channel> {
        Box::new(Self {
            value: self.value.clone(),
            updated: self.updated,
            reducer: Arc::clone(&self.reducer),
        })
    }
}
