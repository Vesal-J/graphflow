use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::GraphError;
use crate::graph::{BranchFunction, NodeFunction};
use crate::hooks::{GraphHook, HookDispatcher, StateContext};

pub struct CompiledGraph<T> {
    pub nodes: HashMap<String, NodeFunction<T>>,
    pub edges: HashMap<String, String>,
    pub conditional_edges: HashMap<String, BranchFunction<T>>,
    pub entry_point: String,
    pub finish_point: String,
    pub dispatcher: HookDispatcher<T>,
}

impl<T> Clone for CompiledGraph<T> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            conditional_edges: self.conditional_edges.clone(),
            entry_point: self.entry_point.clone(),
            finish_point: self.finish_point.clone(),
            dispatcher: self.dispatcher.clone(),
        }
    }
}

impl<T> std::fmt::Debug for CompiledGraph<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledGraph")
            .field("nodes_count", &self.nodes.len())
            .field("edges", &self.edges)
            .field("conditional_edges_count", &self.conditional_edges.len())
            .field("entry_point", &self.entry_point)
            .field("finish_point", &self.finish_point)
            .field("dispatcher", &self.dispatcher)
            .finish()
    }
}

impl<T: 'static> CompiledGraph<T> {
    /// Adds a custom struct implementing `GraphHook<T>` to this compiled graph.
    pub fn add_hook<H>(&mut self, hook: H) -> &mut Self
    where
        H: GraphHook<T> + 'static,
    {
        self.dispatcher.add_hook(Arc::new(hook));
        self
    }

    /// Attaches a hook using builder pattern.
    pub fn with_hook<H>(mut self, hook: H) -> Self
    where
        H: GraphHook<T> + 'static,
    {
        self.add_hook(hook);
        self
    }

    pub fn invoke(&self, mut state: T) -> Result<T, GraphError> {
        let mut current = self.entry_point.clone();
        log::info!("Starting execution of compiled graph at entry point '{current}'");

        self.dispatcher.on_agent_start(&current, &state);

        loop {
            log::info!("NODE: {current}");

            let node = self.nodes.get(&current).ok_or_else(|| {
                log::error!("Node '{current}' not found in compiled graph");
                GraphError::NodeNotFound(current.clone())
            })?;

            self.dispatcher.on_node_start(&current, &state);

            let res = {
                let mut ctx = StateContext::new(&current, &mut state, &self.dispatcher);
                node(&mut ctx)
            };

            self.dispatcher.on_node_end(&current, &state, &res);

            if let Err(e) = res {
                log::error!("Error executing node '{current}': {e}");
                return Err(e);
            }

            if current == self.finish_point {
                log::info!("Reached finish point '{current}', completing graph execution");
                self.dispatcher.on_agent_end(&current, &state);
                break;
            }

            // Conditional edge has priority
            if let Some(branch) = self.conditional_edges.get(&current) {
                self.dispatcher.on_conditional_node_start(&current, &state);
                let next = branch(&state);
                self.dispatcher.on_conditional_node_end(&current, &next, &state);
                log::debug!("Conditional edge evaluated from '{current}' -> '{next}'");
                current = next;
            } else {
                let next = self.edges.get(&current).ok_or_else(|| {
                    log::error!("No outgoing edge found from node '{current}'");
                    GraphError::MissingEdge(current.clone())
                })?.clone();
                log::debug!("Static edge followed from '{current}' -> '{next}'");
                current = next;
            }
        }

        Ok(state)
    }
}
