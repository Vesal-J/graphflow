use std::collections::HashMap;

use crate::errors::GraphError;
use crate::CompiledGraph;

pub type NodeFunction<T> = fn(&mut T) -> Result<(), GraphError>;
pub type BranchFunction<T> = fn(&T) -> String;

pub struct Graph<T> {
    pub nodes: HashMap<String, NodeFunction<T>>,
    pub edges: HashMap<String, String>,
    pub conditional_edges: HashMap<String, BranchFunction<T>>,
    pub entry_point: Option<String>,
    pub finish_point: Option<String>,
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            conditional_edges: HashMap::new(),
            entry_point: None,
            finish_point: None,
        }
    }

    pub fn add_node(&mut self, name: String, function: NodeFunction<T>) -> &mut Self {
        log::debug!("Registering node: '{name}'");
        self.nodes.insert(name, function);
        self
    }

    pub fn add_edge(&mut self, from: String, to: String) -> &mut Self {
        log::debug!("Registering edge: '{from}' -> '{to}'");
        self.edges.insert(from, to);
        self
    }

    pub fn add_conditional_edge(&mut self, from: String, branch: BranchFunction<T>) -> &mut Self {
        log::debug!("Registering conditional edge from '{from}'");
        self.conditional_edges.insert(from, branch);
        self
    }

    pub fn set_entry_point(&mut self, from: String) -> &mut Self {
        log::debug!("Setting entry point: '{from}'");
        self.entry_point = Some(from);
        self
    }

    pub fn set_finish_point(&mut self, from: String) -> &mut Self {
        log::debug!("Setting finish point: '{from}'");
        self.finish_point = Some(from);
        self
    }

    pub fn compile(&mut self) -> Result<CompiledGraph<T>, GraphError> {
        log::info!(
            "Validating and compiling graph with {} node(s), {} edge(s), {} conditional edge(s)",
            self.nodes.len(),
            self.edges.len(),
            self.conditional_edges.len()
        );

        let entry_point_clone = self.entry_point.clone();
        let entry_point = entry_point_clone.ok_or_else(|| {
            log::error!("Graph compilation failed: entry point is not set");
            GraphError::MissingEntryPoint
        })?;

        let finish_point_clone = self.finish_point.clone();
        let finish_point = finish_point_clone.ok_or_else(|| {
            log::error!("Graph compilation failed: finish point is not set");
            GraphError::MissingFinishPoint
        })?;

        for entry in entry_point.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if !self.nodes.contains_key(entry) {
                log::error!("Graph compilation failed: entry point '{entry}' does not exist");
                return Err(GraphError::EntryPointNotFound(entry.to_string()));
            }
        }

        if !self.nodes.contains_key(&finish_point) {
            log::error!("Graph compilation failed: finish point '{finish_point}' does not exist");
            return Err(GraphError::FinishPointNotFound(finish_point));
        }

        for (from, to) in &self.edges {
            if !self.nodes.contains_key(from) {
                log::error!("Graph compilation failed: edge source '{from}' does not exist");
                return Err(GraphError::EdgeSourceNotFound(from.clone()));
            }

            for target in to.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if !self.nodes.contains_key(target) {
                    log::error!("Graph compilation failed: edge target '{target}' does not exist");
                    return Err(GraphError::EdgeTargetNotFound(target.to_string()));
                }
            }
        }

        for from in self.conditional_edges.keys() {
            if !self.nodes.contains_key(from) {
                log::error!("Graph compilation failed: conditional edge source '{from}' does not exist");
                return Err(GraphError::ConditionalEdgeSourceNotFound(from.clone()));
            }
        }

        log::info!(
            "Graph compiled successfully: entry='{entry_point}', finish='{finish_point}'"
        );

        let nodes_clone = self.nodes.clone();
        let edges_clone = self.edges.clone();
        let conditional_edges_clone = self.conditional_edges.clone();

        Ok(CompiledGraph {
            nodes: nodes_clone,
            edges: edges_clone,
            conditional_edges: conditional_edges_clone,
            entry_point,
            finish_point,
        })
    }
}
