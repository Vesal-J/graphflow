use std::sync::Arc;
use crate::errors::GraphError;

/// Trait defining lifecycle hooks for graph execution.
///
/// All methods have default empty implementations, allowing implementors
/// to only handle the lifecycle events they care about.
pub trait GraphHook<T>: Send + Sync {
    /// Called when the graph starts execution at the configured entry point.
    fn on_agent_start(&self, _entry_point: &str, _state: &T) {}

    /// Called after a node successfully completes execution and mutates the state.
    fn on_agent_state_change(&self, _node_name: &str, _state: &T) {}

    /// Called immediately before a node function executes.
    fn on_node_start(&self, _node_name: &str, _state: &T) {}

    /// Called immediately after a node function completes execution, with its `Result`.
    fn on_node_end(&self, _node_name: &str, _state: &T, _result: &Result<(), GraphError>) {}

    /// Called before a conditional edge evaluates its branch function.
    fn on_conditional_node_start(&self, _from_node: &str, _state: &T) {}

    /// Called after a conditional edge evaluates its branch function and selects the next node.
    fn on_conditional_node_end(&self, _from_node: &str, _next_node: &str, _state: &T) {}

    /// Called when the graph reaches the finish point and completes execution.
    fn on_agent_end(&self, _finish_point: &str, _state: &T) {}
}

type AgentStartFn<T> = Arc<dyn Fn(&str, &T) + Send + Sync>;
type AgentStateChangeFn<T> = Arc<dyn Fn(&str, &T) + Send + Sync>;
type NodeStartFn<T> = Arc<dyn Fn(&str, &T) + Send + Sync>;
type NodeEndFn<T> = Arc<dyn Fn(&str, &T, &Result<(), GraphError>) + Send + Sync>;
type ConditionalNodeStartFn<T> = Arc<dyn Fn(&str, &T) + Send + Sync>;
type ConditionalNodeEndFn<T> = Arc<dyn Fn(&str, &str, &T) + Send + Sync>;
type AgentEndFn<T> = Arc<dyn Fn(&str, &T) + Send + Sync>;

/// Closure-based hook container implementing `GraphHook<T>`.
#[derive(Default)]
pub struct Hooks<T> {
    on_agent_start_hooks: Vec<AgentStartFn<T>>,
    on_agent_state_change_hooks: Vec<AgentStateChangeFn<T>>,
    on_node_start_hooks: Vec<NodeStartFn<T>>,
    on_node_end_hooks: Vec<NodeEndFn<T>>,
    on_conditional_node_start_hooks: Vec<ConditionalNodeStartFn<T>>,
    on_conditional_node_end_hooks: Vec<ConditionalNodeEndFn<T>>,
    on_agent_end_hooks: Vec<AgentEndFn<T>>,
}

impl<T> Clone for Hooks<T> {
    fn clone(&self) -> Self {
        Self {
            on_agent_start_hooks: self.on_agent_start_hooks.clone(),
            on_agent_state_change_hooks: self.on_agent_state_change_hooks.clone(),
            on_node_start_hooks: self.on_node_start_hooks.clone(),
            on_node_end_hooks: self.on_node_end_hooks.clone(),
            on_conditional_node_start_hooks: self.on_conditional_node_start_hooks.clone(),
            on_conditional_node_end_hooks: self.on_conditional_node_end_hooks.clone(),
            on_agent_end_hooks: self.on_agent_end_hooks.clone(),
        }
    }
}

impl<T> Hooks<T> {
    /// Creates a new empty `Hooks` container.
    pub fn new() -> Self {
        Self {
            on_agent_start_hooks: Vec::new(),
            on_agent_state_change_hooks: Vec::new(),
            on_node_start_hooks: Vec::new(),
            on_node_end_hooks: Vec::new(),
            on_conditional_node_start_hooks: Vec::new(),
            on_conditional_node_end_hooks: Vec::new(),
            on_agent_end_hooks: Vec::new(),
        }
    }

    /// Registers a hook called when the graph starts execution.
    pub fn add_on_agent_start<F>(&mut self, hook: F) -> &mut Self
    where
        F: Fn(&str, &T) + Send + Sync + 'static,
    {
        self.on_agent_start_hooks.push(Arc::new(hook));
        self
    }

    /// Registers a hook called when state is mutated after a successful node execution.
    pub fn add_on_agent_state_change<F>(&mut self, hook: F) -> &mut Self
    where
        F: Fn(&str, &T) + Send + Sync + 'static,
    {
        self.on_agent_state_change_hooks.push(Arc::new(hook));
        self
    }

    /// Registers a hook called before a node begins execution.
    pub fn add_on_node_start<F>(&mut self, hook: F) -> &mut Self
    where
        F: Fn(&str, &T) + Send + Sync + 'static,
    {
        self.on_node_start_hooks.push(Arc::new(hook));
        self
    }

    /// Registers a hook called after a node finishes execution.
    pub fn add_on_node_end<F>(&mut self, hook: F) -> &mut Self
    where
        F: Fn(&str, &T, &Result<(), GraphError>) + Send + Sync + 'static,
    {
        self.on_node_end_hooks.push(Arc::new(hook));
        self
    }

    /// Registers a hook called before conditional edge evaluation.
    pub fn add_on_conditional_node_start<F>(&mut self, hook: F) -> &mut Self
    where
        F: Fn(&str, &T) + Send + Sync + 'static,
    {
        self.on_conditional_node_start_hooks.push(Arc::new(hook));
        self
    }

    /// Registers a hook called after conditional edge evaluation with the selected target node.
    pub fn add_on_conditional_node_end<F>(&mut self, hook: F) -> &mut Self
    where
        F: Fn(&str, &str, &T) + Send + Sync + 'static,
    {
        self.on_conditional_node_end_hooks.push(Arc::new(hook));
        self
    }

    /// Registers a hook called when the graph completes execution at the finish point.
    pub fn add_on_agent_end<F>(&mut self, hook: F) -> &mut Self
    where
        F: Fn(&str, &T) + Send + Sync + 'static,
    {
        self.on_agent_end_hooks.push(Arc::new(hook));
        self
    }
}

impl<T> GraphHook<T> for Hooks<T> {
    fn on_agent_start(&self, entry_point: &str, state: &T) {
        for hook in &self.on_agent_start_hooks {
            hook(entry_point, state);
        }
    }

    fn on_agent_state_change(&self, node_name: &str, state: &T) {
        for hook in &self.on_agent_state_change_hooks {
            hook(node_name, state);
        }
    }

    fn on_node_start(&self, node_name: &str, state: &T) {
        for hook in &self.on_node_start_hooks {
            hook(node_name, state);
        }
    }

    fn on_node_end(&self, node_name: &str, state: &T, result: &Result<(), GraphError>) {
        for hook in &self.on_node_end_hooks {
            hook(node_name, state, result);
        }
    }

    fn on_conditional_node_start(&self, from_node: &str, state: &T) {
        for hook in &self.on_conditional_node_start_hooks {
            hook(from_node, state);
        }
    }

    fn on_conditional_node_end(&self, from_node: &str, next_node: &str, state: &T) {
        for hook in &self.on_conditional_node_end_hooks {
            hook(from_node, next_node, state);
        }
    }

    fn on_agent_end(&self, finish_point: &str, state: &T) {
        for hook in &self.on_agent_end_hooks {
            hook(finish_point, state);
        }
    }
}

impl<T> std::fmt::Debug for Hooks<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hooks")
            .field("on_agent_start", &self.on_agent_start_hooks.len())
            .field("on_agent_state_change", &self.on_agent_state_change_hooks.len())
            .field("on_node_start", &self.on_node_start_hooks.len())
            .field("on_node_end", &self.on_node_end_hooks.len())
            .field("on_conditional_node_start", &self.on_conditional_node_start_hooks.len())
            .field("on_conditional_node_end", &self.on_conditional_node_end_hooks.len())
            .field("on_agent_end", &self.on_agent_end_hooks.len())
            .finish()
    }
}

/// Internal dispatcher managing multiple `GraphHook<T>` implementations.
#[derive(Default)]
pub struct HookDispatcher<T> {
    hooks: Vec<Arc<dyn GraphHook<T>>>,
}

impl<T> Clone for HookDispatcher<T> {
    fn clone(&self) -> Self {
        Self {
            hooks: self.hooks.clone(),
        }
    }
}

impl<T> HookDispatcher<T> {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add_hook(&mut self, hook: Arc<dyn GraphHook<T>>) {
        self.hooks.push(hook);
    }

    pub fn on_agent_start(&self, entry_point: &str, state: &T) {
        for hook in &self.hooks {
            hook.on_agent_start(entry_point, state);
        }
    }

    pub fn on_agent_state_change(&self, node_name: &str, state: &T) {
        for hook in &self.hooks {
            hook.on_agent_state_change(node_name, state);
        }
    }

    pub fn on_node_start(&self, node_name: &str, state: &T) {
        for hook in &self.hooks {
            hook.on_node_start(node_name, state);
        }
    }

    pub fn on_node_end(&self, node_name: &str, state: &T, result: &Result<(), GraphError>) {
        for hook in &self.hooks {
            hook.on_node_end(node_name, state, result);
        }
    }

    pub fn on_conditional_node_start(&self, from_node: &str, state: &T) {
        for hook in &self.hooks {
            hook.on_conditional_node_start(from_node, state);
        }
    }

    pub fn on_conditional_node_end(&self, from_node: &str, next_node: &str, state: &T) {
        for hook in &self.hooks {
            hook.on_conditional_node_end(from_node, next_node, state);
        }
    }

    pub fn on_agent_end(&self, finish_point: &str, state: &T) {
        for hook in &self.hooks {
            hook.on_agent_end(finish_point, state);
        }
    }
}

impl<T> std::fmt::Debug for HookDispatcher<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookDispatcher")
            .field("hooks_count", &self.hooks.len())
            .finish()
    }
}

/// Context provided to node functions during execution.
///
/// Direct mutation of the underlying state is prevented. To mutate state,
/// call [`update`](Self::update) or [`set`](Self::set), which automatically
/// trigger the `on_agent_state_change` lifecycle hook.
pub struct StateContext<'a, T> {
    node_name: &'a str,
    state: &'a mut T,
    dispatcher: &'a HookDispatcher<T>,
}

impl<'a, T> StateContext<'a, T> {
    /// Creates a new `StateContext`.
    pub fn new(node_name: &'a str, state: &'a mut T, dispatcher: &'a HookDispatcher<T>) -> Self {
        Self {
            node_name,
            state,
            dispatcher,
        }
    }

    /// Returns a read-only reference to the current state.
    pub fn get(&self) -> &T {
        self.state
    }

    /// Returns the name of the currently executing node.
    pub fn node_name(&self) -> &str {
        self.node_name
    }

    /// Mutates the state via a closure and triggers `on_agent_state_change`.
    pub fn update<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let ret = f(self.state);
        self.dispatcher.on_agent_state_change(self.node_name, self.state);
        ret
    }

    /// Replaces the state with a new value and triggers `on_agent_state_change`.
    pub fn set(&mut self, new_state: T) {
        *self.state = new_state;
        self.dispatcher.on_agent_state_change(self.node_name, self.state);
    }
}

impl<'a, T> std::ops::Deref for StateContext<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for StateContext<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateContext")
            .field("node_name", &self.node_name)
            .field("state", &self.state)
            .finish()
    }
}

