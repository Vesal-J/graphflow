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

    // 11. Graph::default works identically to Graph::new
    #[test]
    fn test_graph_default() {
        let graph: Graph<TestState> = Graph::default();

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.conditional_edges.is_empty());
        assert!(graph.entry_point.is_none());
        assert!(graph.finish_point.is_none());
    }

    // 12. Overwriting an existing node replaces the function
    #[test]
    fn test_overwrite_node() {
        let mut graph: Graph<TestState> = Graph::new();

        graph.add_node("action".to_string(), increment);
        assert_eq!(graph.nodes.len(), 1);

        graph.add_node("action".to_string(), increment_by_two);
        assert_eq!(graph.nodes.len(), 1);
    }

    // 13. Overwriting an existing edge replaces the target
    #[test]
    fn test_overwrite_edge() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_edge("a".to_string(), "b".to_string())
            .add_edge("a".to_string(), "c".to_string());

        assert_eq!(graph.edges.get("a"), Some(&"c".to_string()));
    }

    // 14. Overwriting a conditional edge replaces the branch function
    #[test]
    fn test_overwrite_conditional_edge() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_conditional_edge("a".to_string(), |_| "b".to_string())
            .add_conditional_edge("a".to_string(), |_| "c".to_string());

        let branch_fn = graph.conditional_edges.get("a").unwrap();
        let state = TestState { count: 0 };
        assert_eq!(branch_fn(&state), "c");
    }

    // 15. Compile fails if finish point node does not exist
    #[test]
    fn test_compile_with_invalid_finish_point_fails() {
        let mut graph: Graph<TestState> = Graph::new();

        graph.add_node("start".to_string(), increment);

        let result = graph
            .set_entry_point("start".to_string())
            .set_finish_point("non_existent_finish".to_string())
            .compile();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Finish point 'non_existent_finish' does not exist"
        );
    }

    // 16. Compile fails if an edge source does not exist
    #[test]
    fn test_compile_with_invalid_edge_source_fails() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("start".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_edge("ghost_source".to_string(), "finish".to_string())
            .set_entry_point("start".to_string())
            .set_finish_point("finish".to_string());

        let result = graph.compile();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Edge source 'ghost_source' does not exist"
        );
    }

    // 17. Compile fails if a conditional edge source does not exist
    #[test]
    fn test_compile_with_invalid_conditional_edge_source_fails() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("start".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_conditional_edge("ghost_source".to_string(), |_| "finish".to_string())
            .set_entry_point("start".to_string())
            .set_finish_point("finish".to_string());

        let result = graph.compile();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Conditional edge source 'ghost_source' does not exist"
        );
    }

    // 18. Graph can be compiled multiple times
    #[test]
    fn test_compile_can_be_called_multiple_times() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("start".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_edge("start".to_string(), "finish".to_string())
            .set_entry_point("start".to_string())
            .set_finish_point("finish".to_string());

        let compiled_one = graph.compile();
        let compiled_two = graph.compile();

        assert!(compiled_one.is_ok());
        assert!(compiled_two.is_ok());
    }

    // 19. Single-node graph where entry point equals finish point
    #[test]
    fn test_single_node_graph() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct ExecutionLog {
            log: Arc<Mutex<Vec<String>>>,
        }

        let mut graph: Graph<ExecutionLog> = Graph::new();

        graph
            .add_node("only_node".to_string(), |state| {
                state.log.lock().unwrap().push("only_node_ran".to_string());
                Ok(())
            })
            .set_entry_point("only_node".to_string())
            .set_finish_point("only_node".to_string());

        let compiled = graph.compile().unwrap();

        let state = ExecutionLog {
            log: Arc::new(Mutex::new(Vec::new())),
        };

        compiled.invoke(state.clone()).unwrap();

        let visited = state.log.lock().unwrap().clone();
        assert_eq!(visited, vec!["only_node_ran"]);
    }

    // 20. Exact order of node execution and state tracking
    #[test]
    fn test_exact_execution_order_and_state_history() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct OrderState {
            history: Arc<Mutex<Vec<&'static str>>>,
        }

        let mut graph: Graph<OrderState> = Graph::new();

        graph
            .add_node("step_a".to_string(), |s| {
                s.history.lock().unwrap().push("A");
                Ok(())
            })
            .add_node("step_b".to_string(), |s| {
                s.history.lock().unwrap().push("B");
                Ok(())
            })
            .add_node("step_c".to_string(), |s| {
                s.history.lock().unwrap().push("C");
                Ok(())
            })
            .add_edge("step_a".to_string(), "step_b".to_string())
            .add_edge("step_b".to_string(), "step_c".to_string())
            .set_entry_point("step_a".to_string())
            .set_finish_point("step_c".to_string());

        let compiled = graph.compile().unwrap();

        let state = OrderState {
            history: Arc::new(Mutex::new(Vec::new())),
        };

        compiled.invoke(state.clone()).unwrap();

        let order = state.history.lock().unwrap().clone();
        assert_eq!(order, vec!["A", "B", "C"]);
    }

    // 21. Conditional edge takes priority over static edge
    #[test]
    fn test_conditional_edge_priority_over_static_edge() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct PriorityState {
            visited: Arc<Mutex<Vec<&'static str>>>,
        }

        let mut graph: Graph<PriorityState> = Graph::new();

        graph
            .add_node("start".to_string(), |s| {
                s.visited.lock().unwrap().push("start");
                Ok(())
            })
            .add_node("static_target".to_string(), |s| {
                s.visited.lock().unwrap().push("static_target");
                Ok(())
            })
            .add_node("conditional_target".to_string(), |s| {
                s.visited.lock().unwrap().push("conditional_target");
                Ok(())
            })
            // Both static edge and conditional edge from "start"
            .add_edge("start".to_string(), "static_target".to_string())
            .add_conditional_edge("start".to_string(), |_| "conditional_target".to_string())
            .add_edge("static_target".to_string(), "conditional_target".to_string())
            .set_entry_point("start".to_string())
            .set_finish_point("conditional_target".to_string());

        let compiled = graph.compile().unwrap();

        let state = PriorityState {
            visited: Arc::new(Mutex::new(Vec::new())),
        };

        compiled.invoke(state.clone()).unwrap();

        let visited = state.visited.lock().unwrap().clone();
        // Conditional edge must be chosen directly, bypassing static_target
        assert_eq!(visited, vec!["start", "conditional_target"]);
    }

    // 22. Multi-branch conditional routing with multiple targets
    #[test]
    fn test_multi_branch_routing() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct RouteState {
            selector: usize,
            route_taken: Arc<Mutex<Option<&'static str>>>,
        }

        fn build_graph() -> crate::CompiledGraph<RouteState> {
            let mut graph: Graph<RouteState> = Graph::new();
            graph
                .add_node("decide".to_string(), |_| Ok(()))
                .add_node("path_0".to_string(), |s| {
                    *s.route_taken.lock().unwrap() = Some("path_0");
                    Ok(())
                })
                .add_node("path_1".to_string(), |s| {
                    *s.route_taken.lock().unwrap() = Some("path_1");
                    Ok(())
                })
                .add_node("path_default".to_string(), |s| {
                    *s.route_taken.lock().unwrap() = Some("path_default");
                    Ok(())
                })
                .add_node("finish".to_string(), |_| Ok(()))
                .add_conditional_edge("decide".to_string(), |s| match s.selector {
                    0 => "path_0".to_string(),
                    1 => "path_1".to_string(),
                    _ => "path_default".to_string(),
                })
                .add_edge("path_0".to_string(), "finish".to_string())
                .add_edge("path_1".to_string(), "finish".to_string())
                .add_edge("path_default".to_string(), "finish".to_string())
                .set_entry_point("decide".to_string())
                .set_finish_point("finish".to_string());
            graph.compile().unwrap()
        }

        let compiled = build_graph();

        for (selector, expected) in [(0, "path_0"), (1, "path_1"), (99, "path_default")] {
            let state = RouteState {
                selector,
                route_taken: Arc::new(Mutex::new(None)),
            };
            compiled.invoke(state.clone()).unwrap();
            assert_eq!(*state.route_taken.lock().unwrap(), Some(expected));
        }
    }

    // 23. Node function returning error propagates to invoke
    #[test]
    fn test_node_error_propagates() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("failing_node".to_string(), |_| Err(Error))
            .set_entry_point("failing_node".to_string())
            .set_finish_point("failing_node".to_string());

        let compiled = graph.compile().unwrap();
        let result = compiled.invoke(TestState { count: 0 });

        assert!(result.is_err());
    }

    // 24. Node error halts subsequent execution
    #[test]
    fn test_node_error_halts_execution() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct HaltState {
            ran_second: Arc<Mutex<bool>>,
        }

        let mut graph: Graph<HaltState> = Graph::new();

        graph
            .add_node("first_node_fails".to_string(), |_| Err(Error))
            .add_node("second_node".to_string(), |s| {
                *s.ran_second.lock().unwrap() = true;
                Ok(())
            })
            .add_edge("first_node_fails".to_string(), "second_node".to_string())
            .set_entry_point("first_node_fails".to_string())
            .set_finish_point("second_node".to_string());

        let compiled = graph.compile().unwrap();
        let state = HaltState {
            ran_second: Arc::new(Mutex::new(false)),
        };

        let result = compiled.invoke(state.clone());
        assert!(result.is_err());
        assert_eq!(*state.ran_second.lock().unwrap(), false);
    }

    // 25. Reaching a node without an outgoing edge (when not finish point) returns Error
    #[test]
    fn test_missing_outgoing_edge_at_runtime_returns_error() {
        let mut graph: Graph<TestState> = Graph::new();

        // "dead_end" is not the finish point and has no edge leading to "finish"
        graph
            .add_node("start".to_string(), increment)
            .add_node("dead_end".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_edge("start".to_string(), "dead_end".to_string())
            .set_entry_point("start".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();
        let result = compiled.invoke(TestState { count: 0 });

        assert!(result.is_err());
    }

    // 26. Conditional edge routing to a nonexistent node at runtime returns Error
    #[test]
    fn test_conditional_edge_routing_to_nonexistent_node_fails_runtime() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("start".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_conditional_edge("start".to_string(), |_| "phantom_node".to_string())
            .set_entry_point("start".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();
        let result = compiled.invoke(TestState { count: 0 });

        assert!(result.is_err());
    }

    // 27. Exact loop counter verification
    #[test]
    fn test_exact_loop_iteration_count() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        #[derive(Clone)]
        struct CounterState {
            loop_count: Arc<AtomicUsize>,
        }

        let mut graph: Graph<CounterState> = Graph::new();

        graph
            .add_node("loop_body".to_string(), |s| {
                s.loop_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .add_node("finish".to_string(), |_| Ok(()))
            .add_conditional_edge("loop_body".to_string(), |s| {
                if s.loop_count.load(Ordering::SeqCst) < 5 {
                    "loop_body".to_string()
                } else {
                    "finish".to_string()
                }
            })
            .set_entry_point("loop_body".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();
        let state = CounterState {
            loop_count: Arc::new(AtomicUsize::new(0)),
        };

        compiled.invoke(state.clone()).unwrap();
        assert_eq!(state.loop_count.load(Ordering::SeqCst), 5);
    }

    // 28. Diamond graph topology execution
    #[test]
    fn test_diamond_graph_topology() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct DiamondState {
            route_left: bool,
            path: Arc<Mutex<Vec<&'static str>>>,
        }

        let mut graph: Graph<DiamondState> = Graph::new();

        graph
            .add_node("split".to_string(), |s| {
                s.path.lock().unwrap().push("split");
                Ok(())
            })
            .add_node("left".to_string(), |s| {
                s.path.lock().unwrap().push("left");
                Ok(())
            })
            .add_node("right".to_string(), |s| {
                s.path.lock().unwrap().push("right");
                Ok(())
            })
            .add_node("join".to_string(), |s| {
                s.path.lock().unwrap().push("join");
                Ok(())
            })
            .add_node("finish".to_string(), |s| {
                s.path.lock().unwrap().push("finish");
                Ok(())
            })
            .add_conditional_edge("split".to_string(), |s| {
                if s.route_left {
                    "left".to_string()
                } else {
                    "right".to_string()
                }
            })
            .add_edge("left".to_string(), "join".to_string())
            .add_edge("right".to_string(), "join".to_string())
            .add_edge("join".to_string(), "finish".to_string())
            .set_entry_point("split".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();

        // Test Left path
        let left_state = DiamondState {
            route_left: true,
            path: Arc::new(Mutex::new(Vec::new())),
        };
        compiled.invoke(left_state.clone()).unwrap();
        assert_eq!(
            *left_state.path.lock().unwrap(),
            vec!["split", "left", "join", "finish"]
        );

        // Test Right path
        let right_state = DiamondState {
            route_left: false,
            path: Arc::new(Mutex::new(Vec::new())),
        };
        compiled.invoke(right_state.clone()).unwrap();
        assert_eq!(
            *right_state.path.lock().unwrap(),
            vec!["split", "right", "join", "finish"]
        );
    }
}

