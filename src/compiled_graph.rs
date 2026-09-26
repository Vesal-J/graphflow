use std::{collections::HashMap, fmt::Error};

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
    pub fn invoke(&self, mut state: T) -> Result<T, Error> {
        let mut current = self.entry_point.clone();

        loop {
            println!("NODE: {}", current);

            let node = self.nodes.get(&current).ok_or(Error)?;

            node(&mut state)?;

            if current == self.finish_point {
                break;
            }

            // Conditional edge has priority
            if let Some(branch) = self.conditional_edges.get(&current) {
                current = branch(&state);
            } else {
                current = self.edges.get(&current).ok_or(Error)?.clone();
            }
        }

        Ok(state)
    }
}
