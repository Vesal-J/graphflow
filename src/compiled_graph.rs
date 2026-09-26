use std::collections::HashMap;

use crate::errors::GraphError;
use crate::graph::{BranchFunction, NodeFunction};

#[derive(Debug)]
pub struct CompiledGraph<T> {
    pub nodes: HashMap<String, NodeFunction<T>>,
    pub edges: HashMap<String, String>,
    pub conditional_edges: HashMap<String, BranchFunction<T>>,
    pub entry_point: String,
    pub finish_point: String,
}

impl<T> CompiledGraph<T> {
    pub fn invoke(&self, mut state: T) -> Result<T, GraphError> {
        let mut current = self.entry_point.clone();
        log::info!("Starting execution of compiled graph at entry point '{current}'");

        loop {
            log::info!("NODE: {current}");

            let node = self.nodes.get(&current).ok_or_else(|| {
                log::error!("Node '{current}' not found in compiled graph");
                GraphError::NodeNotFound(current.clone())
            })?;

            if let Err(e) = node(&mut state) {
                log::error!("Error executing node '{current}': {e}");
                return Err(e);
            }

            if current == self.finish_point {
                log::info!("Reached finish point '{current}', completing graph execution");
                break;
            }

            // Conditional edge has priority
            if let Some(branch) = self.conditional_edges.get(&current) {
                let next = branch(&state);
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
