use std::{collections::HashMap, fmt::Error};

use crate::CompiledGraph;

pub type NodeFunction<T> = fn(&mut T) -> Result<(), Error>;
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
        self.nodes.insert(name, function);
        self
    }

    pub fn add_edge(&mut self, from: String, to: String) -> &mut Self {
        self.edges.insert(from, to);
        self
    }

    pub fn add_parallel_edge(&mut self, from: String, targets: Vec<String>) -> &mut Self {
        self.edges.insert(from, targets.join(", "));
        self
    }

    pub fn add_conditional_edge(&mut self, from: String, branch: BranchFunction<T>) -> &mut Self {
        self.conditional_edges.insert(from, branch);
        self
    }

    pub fn set_entry_point(&mut self, from: String) -> &mut Self {
        self.entry_point = Some(from);
        self
    }

    pub fn set_finish_point(&mut self, from: String) -> &mut Self {
        self.finish_point = Some(from);
        self
    }

    pub fn compile(&mut self) -> Result<CompiledGraph<T>, String> {
        let entry_point_clone = self.entry_point.clone();
        let entry_point = entry_point_clone.ok_or("Entry point is not set")?;

        let entry_point_clone = self.finish_point.clone();
        let finish_point = entry_point_clone.ok_or("Finish point is not set")?;

        for entry in entry_point.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if !self.nodes.contains_key(entry) {
                return Err(format!("Entry point '{}' does not exist", entry));
            }
        }

        if !self.nodes.contains_key(&finish_point) {
            return Err(format!("Finish point '{}' does not exist", finish_point));
        }

        for (from, to) in &self.edges {
            if !self.nodes.contains_key(from) {
                return Err(format!("Edge source '{}' does not exist", from));
            }

            for target in to.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if !self.nodes.contains_key(target) {
                    return Err(format!("Edge target '{}' does not exist", target));
                }
            }
        }

        for from in self.conditional_edges.keys() {
            if !self.nodes.contains_key(from) {
                return Err(format!("Conditional edge source '{}' does not exist", from));
            }
        }

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
