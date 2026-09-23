#[cfg(test)]
mod tests {
    use std::fmt::Error;

    use crate::Graph;

    #[derive(Debug, PartialEq)]
    struct TestState {
        count: usize,
    }

    fn increment(state: &mut TestState) -> Result<(), Error> {
        state.count += 1;
        Ok(())
    }

    fn increment_by_two(state: &mut TestState) -> Result<(), Error> {
        state.count += 2;
        Ok(())
    }

    fn finish(_state: &mut TestState) -> Result<(), Error> {
        Ok(())
    }

    // 1. Graph can be created
    #[test]
    fn test_graph_new() {
        let graph: Graph<TestState> = Graph::new();

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.conditional_edges.is_empty());
        assert!(graph.entry_point.is_none());
        assert!(graph.finish_point.is_none());
    }

    // 2. Adding a node works
    #[test]
    fn test_add_node() {
        let mut graph = Graph::new();

        graph.add_node("increment".to_string(), increment);

        assert!(graph.nodes.contains_key("increment"));
        assert_eq!(graph.nodes.len(), 1);
    }

    // 3. Adding normal edges works
    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new();

        graph
            .add_node("a".to_string(), increment)
            .add_node("b".to_string(), increment)
            .add_edge("a".to_string(), "b".to_string());

        assert_eq!(graph.edges.get("a"), Some(&"b".to_string()));
    }

    // 4. Conditional edges are stored correctly
    #[test]
    fn test_add_conditional_edge() {
        let mut graph = Graph::new();

        graph
            .add_node("a".to_string(), increment)
            .add_node("b".to_string(), increment)
            .add_conditional_edge("a".to_string(), |_state| "b".to_string());

        assert!(graph.conditional_edges.contains_key("a"));
    }

    // 5. Compile fails if entry point is missing
    #[test]
    fn test_compile_without_entry_point_fails() {
        let mut graph = Graph::new();

        graph.add_node("finish".to_string(), finish);

        let result = graph.set_finish_point("finish".to_string()).compile();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Entry point is not set");
    }

    // 6. Compile fails if finish point is missing
    #[test]
    fn test_compile_without_finish_point_fails() {
        let mut graph = Graph::new();

        graph.add_node("start".to_string(), increment);

        let result = graph.set_entry_point("start".to_string()).compile();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Finish point is not set");
    }

    // 7. Compile fails if entry point doesn't exist
    #[test]
    fn test_compile_with_invalid_entry_point_fails() {
        let mut graph = Graph::new();

        graph.add_node("finish".to_string(), finish);

        let result = graph
            .set_entry_point("does_not_exist".to_string())
            .set_finish_point("finish".to_string())
            .compile();

        assert!(result.is_err());

        assert_eq!(
            result.unwrap_err(),
            "Entry point 'does_not_exist' does not exist"
        );
    }

    // 8. Compile fails if an edge points to a nonexistent node
    #[test]
    fn test_compile_with_invalid_edge_target_fails() {
        let mut graph = Graph::new();

        graph
            .add_node("start".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_edge("start".to_string(), "does_not_exist".to_string())
            .set_entry_point("start".to_string())
            .set_finish_point("finish".to_string());

        let result = graph.compile();

        assert!(result.is_err());

        assert_eq!(
            result.unwrap_err(),
            "Edge target 'does_not_exist' does not exist"
        );
    }

    // 9. Graph executes nodes in the correct order
    #[test]
    fn test_graph_executes_nodes() {
        let mut graph = Graph::new();

        graph
            .add_node("increment".to_string(), increment)
            .add_node("increment2".to_string(), increment_by_two)
            .add_node("finish".to_string(), finish)
            .add_edge("increment".to_string(), "increment2".to_string())
            .add_edge("increment2".to_string(), "finish".to_string())
            .set_entry_point("increment".to_string())
            .set_finish_point("finish".to_string());

        let graph = graph.compile().unwrap();

        let state = TestState { count: 0 };

        // invoke currently returns ()
        graph.invoke(state).unwrap();

        // This test verifies that execution doesn't fail,
        // but to verify final state we should eventually
        // make invoke return the state.
    }

    // 10. Conditional routing chooses the correct branch
    #[test]
    fn test_conditional_routing() {
        let mut graph = Graph::new();

        graph
            .add_node("increment".to_string(), increment)
            .add_node("router".to_string(), increment_by_two)
            .add_node("finish".to_string(), finish)
            .add_edge("increment".to_string(), "router".to_string())
            .add_conditional_edge("router".to_string(), |state| {
                if state.count < 5 {
                    "increment".to_string()
                } else {
                    "finish".to_string()
                }
            })
            .set_entry_point("increment".to_string())
            .set_finish_point("finish".to_string());

        let graph = graph.compile().unwrap();

        let state = TestState { count: 0 };

        graph.invoke(state).unwrap();
    }
}
