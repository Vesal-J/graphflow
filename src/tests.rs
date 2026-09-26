#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::{Graph, GraphError, StateContext};

    #[derive(Debug, PartialEq, Clone)]
    struct TestState {
        count: usize,
    }

    fn increment(ctx: &mut StateContext<'_, TestState>) -> Result<(), GraphError> {
        ctx.update(|s| s.count += 1);
        Ok(())
    }

    fn increment_by_two(ctx: &mut StateContext<'_, TestState>) -> Result<(), GraphError> {
        ctx.update(|s| s.count += 2);
        Ok(())
    }

    fn finish(_ctx: &mut StateContext<'_, TestState>) -> Result<(), GraphError> {
        Ok(())
    }

    // Graph can be created
    #[test]
    fn test_graph_new() {
        let graph: Graph<TestState> = Graph::new();

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.conditional_edges.is_empty());
        assert!(graph.entry_point.is_none());
        assert!(graph.finish_point.is_none());
    }

    // Adding a node works
    #[test]
    fn test_add_node() {
        let mut graph = Graph::new();

        graph.add_node("increment".to_string(), increment);

        assert!(graph.nodes.contains_key("increment"));
        assert_eq!(graph.nodes.len(), 1);
    }

    // Adding normal edges works
    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new();

        graph
            .add_node("a".to_string(), increment)
            .add_node("b".to_string(), increment)
            .add_edge("a".to_string(), "b".to_string());

        assert_eq!(graph.edges.get("a"), Some(&"b".to_string()));
    }

    // Conditional edges are stored correctly
    #[test]
    fn test_add_conditional_edge() {
        let mut graph = Graph::new();

        graph
            .add_node("a".to_string(), increment)
            .add_node("b".to_string(), increment)
            .add_conditional_edge("a".to_string(), |_state| "b".to_string());

        assert!(graph.conditional_edges.contains_key("a"));
    }

    // Compile fails if entry point is missing
    #[test]
    fn test_compile_without_entry_point_fails() {
        let mut graph = Graph::new();

        graph.add_node("finish".to_string(), finish);

        let result = graph.set_finish_point("finish".to_string()).compile();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Entry point is not set");
    }

    // Compile fails if finish point is missing
    #[test]
    fn test_compile_without_finish_point_fails() {
        let mut graph = Graph::new();

        graph.add_node("start".to_string(), increment);

        let result = graph.set_entry_point("start".to_string()).compile();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Finish point is not set");
    }

    // Compile fails if entry point doesn't exist
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

    // Compile fails if an edge points to a nonexistent node
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

    // Graph executes nodes in the correct order
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

    // Conditional routing chooses the correct branch
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

    // Graph::default works identically to Graph::new
    #[test]
    fn test_graph_default() {
        let graph: Graph<TestState> = Graph::default();

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.conditional_edges.is_empty());
        assert!(graph.entry_point.is_none());
        assert!(graph.finish_point.is_none());
    }

    // Overwriting an existing node replaces the function
    #[test]
    fn test_overwrite_node() {
        let mut graph: Graph<TestState> = Graph::new();

        graph.add_node("action".to_string(), increment);
        assert_eq!(graph.nodes.len(), 1);

        graph.add_node("action".to_string(), increment_by_two);
        assert_eq!(graph.nodes.len(), 1);
    }

    // Overwriting an existing edge replaces the target
    #[test]
    fn test_overwrite_edge() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_edge("a".to_string(), "b".to_string())
            .add_edge("a".to_string(), "c".to_string());

        assert_eq!(graph.edges.get("a"), Some(&"c".to_string()));
    }

    // Overwriting a conditional edge replaces the branch function
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

    // Compile fails if finish point node does not exist
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

    // Compile fails if an edge source does not exist
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

    // Compile fails if a conditional edge source does not exist
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

    // Graph can be compiled multiple times
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

    // Single-node graph where entry point equals finish point
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

    // Exact order of node execution and state tracking
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

    // Conditional edge takes priority over static edge
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
            .add_edge(
                "static_target".to_string(),
                "conditional_target".to_string(),
            )
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

    // Multi-branch conditional routing with multiple targets
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

    // Node function returning error propagates to invoke
    #[test]
    fn test_node_error_propagates() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("failing_node".to_string(), |_| {
                Err(GraphError::ExecutionError("Intentional failure".to_string()))
            })
            .set_entry_point("failing_node".to_string())
            .set_finish_point("failing_node".to_string());

        let compiled = graph.compile().unwrap();
        let result = compiled.invoke(TestState { count: 0 });

        assert!(result.is_err());
    }

    // Node error halts subsequent execution
    #[test]
    fn test_node_error_halts_execution() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct HaltState {
            ran_second: Arc<Mutex<bool>>,
        }

        let mut graph: Graph<HaltState> = Graph::new();

        graph
            .add_node("first_node_fails".to_string(), |_| {
                Err(GraphError::ExecutionError("Intentional failure".to_string()))
            })
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
        assert!(!(*state.ran_second.lock().unwrap()));
    }

    // Reaching a node without an outgoing edge (when not finish point) returns Error
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

    // Conditional edge routing to a nonexistent node at runtime returns Error
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

    // Exact loop counter verification
    #[test]
    fn test_exact_loop_iteration_count() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

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

    // Diamond graph topology execution
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

    // invoke_with_state returns the final mutated state directly
    #[test]
    fn test_pregel_invoke_with_state() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("step_1".to_string(), increment)
            .add_node("step_2".to_string(), increment_by_two)
            .add_node("finish".to_string(), finish)
            .add_edge("step_1".to_string(), "step_2".to_string())
            .add_edge("step_2".to_string(), "finish".to_string())
            .set_entry_point("step_1".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();
        let final_state = compiled.invoke(TestState { count: 10 }).unwrap();

        assert_eq!(final_state.count, 13);
    }

    // invoke_pregel explicit method works identically
    #[test]
    fn test_pregel_invoke_pregel_explicit() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("increment".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_edge("increment".to_string(), "finish".to_string())
            .set_entry_point("increment".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();
        let result = compiled.invoke(TestState { count: 0 });

        assert!(result.is_ok());
    }

    // Compile fails when one of the parallel entry points does not exist
    #[test]
    fn test_compile_with_invalid_parallel_entry_point_fails() {
        let mut graph: Graph<TestState> = Graph::new();

        graph
            .add_node("valid_entry".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .set_entry_point("valid_entry, ghost_entry".to_string())
            .set_finish_point("finish".to_string());

        let result = graph.compile();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Entry point 'ghost_entry' does not exist"
        );
    }

    // Custom GraphError variant matching and display checks
    #[test]
    fn test_graph_error_variants_and_display() {
        use crate::errors::{ChannelError, GraphError, GraphflowError, PregelError};
        use std::error::Error;

        let err_entry = GraphError::MissingEntryPoint;
        assert_eq!(err_entry.to_string(), "Entry point is not set");
        assert_eq!(err_entry, "Entry point is not set");

        let err_finish = GraphError::MissingFinishPoint;
        assert_eq!(err_finish.to_string(), "Finish point is not set");

        let err_entry_not_found = GraphError::EntryPointNotFound("start".to_string());
        assert_eq!(err_entry_not_found, "Entry point 'start' does not exist");

        let err_finish_not_found = GraphError::FinishPointNotFound("end".to_string());
        assert_eq!(err_finish_not_found, "Finish point 'end' does not exist");

        let err_edge_src = GraphError::EdgeSourceNotFound("ghost".to_string());
        assert_eq!(err_edge_src, "Edge source 'ghost' does not exist");

        let err_edge_dst = GraphError::EdgeTargetNotFound("ghost".to_string());
        assert_eq!(err_edge_dst, "Edge target 'ghost' does not exist");

        let err_cond = GraphError::ConditionalEdgeSourceNotFound("ghost".to_string());
        assert_eq!(
            err_cond,
            "Conditional edge source 'ghost' does not exist"
        );

        let err_node = GraphError::NodeNotFound("missing".to_string());
        assert_eq!(err_node, "Node 'missing' does not exist");

        let err_missing_edge = GraphError::MissingEdge("dead_end".to_string());
        assert_eq!(
            err_missing_edge,
            "No outgoing edge found from node 'dead_end'"
        );

        let err_exec = GraphError::ExecutionError("kaboom".to_string());
        assert_eq!(err_exec, "Node execution failed: kaboom");

        // From conversions
        let from_str: GraphError = "custom error".into();
        assert_eq!(from_str, GraphError::ExecutionError("custom error".to_string()));

        let from_string: GraphError = String::from("custom string").into();
        assert_eq!(from_string, GraphError::ExecutionError("custom string".to_string()));

        let from_fmt: GraphError = std::fmt::Error.into();
        assert!(matches!(from_fmt, GraphError::ExecutionError(_)));

        // ChannelError checks
        let ch_err = ChannelError::TypeMismatch("expected i32".to_string());
        assert_eq!(ch_err.to_string(), "Type mismatch in channel: expected i32");
        let pregel_err: PregelError = ch_err.into();
        assert_eq!(pregel_err, PregelError::TypeMismatch("expected i32".to_string()));

        let ch_custom = ChannelError::Custom("something failed".to_string());
        let pregel_custom: PregelError = ch_custom.clone().into();
        assert_eq!(pregel_custom, PregelError::Channel(ch_custom));
        assert!(pregel_custom.source().is_some());

        // GraphflowError checks
        let top_graph: GraphflowError = GraphError::MissingEntryPoint.into();
        assert_eq!(top_graph.to_string(), "Entry point is not set");
        assert!(top_graph.source().is_some());

        let top_pregel: GraphflowError = PregelError::EmptyInput.into();
        assert_eq!(top_pregel.to_string(), "Input is empty or invalid");
        assert!(top_pregel.source().is_some());
    }

    // Typed error matching during compilation and execution
    #[test]
    fn test_typed_error_matching() {
        let mut graph: Graph<TestState> = Graph::new();
        let err = graph.compile().unwrap_err();
        assert_eq!(err, GraphError::MissingEntryPoint);

        graph.set_entry_point("a".to_string());
        let err2 = graph.compile().unwrap_err();
        assert_eq!(err2, GraphError::MissingFinishPoint);

        graph.set_finish_point("b".to_string());
        let err3 = graph.compile().unwrap_err();
        assert_eq!(err3, GraphError::EntryPointNotFound("a".to_string()));
    }

    // Logging system initialization and public re-exports test
    #[test]
    fn test_logging_init() {
        use crate::logging::{init, init_with_level, try_init, try_init_with_level, LevelFilter};
        use crate::{init_logger, init_logger_with_level};

        // Calling init or try_init should not panic even if called multiple times
        init();
        init_with_level(LevelFilter::Debug);
        init_logger();
        init_logger_with_level(LevelFilter::Trace);

        let res_try = try_init();
        assert!(res_try.is_err(), "try_init should return Err once logger is initialized");

        let res_try_level = try_init_with_level(LevelFilter::Info);
        assert!(res_try_level.is_err(), "try_init_with_level should return Err once logger is initialized");

        log::trace!("Test trace statement from unit test");
        log::debug!("Test log statement from unit test");
        log::info!("Test info statement from unit test");
        log::warn!("Test warn statement from unit test");
        log::error!("Test error statement from unit test");
    }

    // Hook system tests
    #[test]
    fn test_hooks_full_lifecycle() {
        use std::sync::{Arc, Mutex};

        #[derive(Debug, PartialEq, Clone)]
        enum HookEvent {
            AgentStart(String, usize),
            NodeStart(String, usize),
            NodeEnd(String, usize, bool),
            StateChange(String, usize),
            ConditionalStart(String, usize),
            ConditionalEnd(String, String, usize),
            AgentEnd(String, usize),
        }

        let events: Arc<Mutex<Vec<HookEvent>>> = Arc::new(Mutex::new(Vec::new()));

        let mut graph: Graph<TestState> = Graph::new();

        let e1 = events.clone();
        graph.on_agent_start(move |entry, state| {
            e1.lock().unwrap().push(HookEvent::AgentStart(entry.to_string(), state.count));
        });

        let e2 = events.clone();
        graph.on_node_start(move |node, state| {
            e2.lock().unwrap().push(HookEvent::NodeStart(node.to_string(), state.count));
        });

        let e3 = events.clone();
        graph.on_node_end(move |node, state, result| {
            e3.lock().unwrap().push(HookEvent::NodeEnd(node.to_string(), state.count, result.is_ok()));
        });

        let e4 = events.clone();
        graph.on_agent_state_change(move |node, state| {
            e4.lock().unwrap().push(HookEvent::StateChange(node.to_string(), state.count));
        });

        let e5 = events.clone();
        graph.on_conditional_node_start(move |node, state| {
            e5.lock().unwrap().push(HookEvent::ConditionalStart(node.to_string(), state.count));
        });

        let e6 = events.clone();
        graph.on_conditional_node_end(move |from, next, state| {
            e6.lock().unwrap().push(HookEvent::ConditionalEnd(from.to_string(), next.to_string(), state.count));
        });

        let e7 = events.clone();
        graph.on_agent_end(move |finish, state| {
            e7.lock().unwrap().push(HookEvent::AgentEnd(finish.to_string(), state.count));
        });

        graph
            .add_node("step_1".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_conditional_edge("step_1".to_string(), |_| "finish".to_string())
            .set_entry_point("step_1".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();
        let final_state = compiled.invoke(TestState { count: 10 }).unwrap();
        assert_eq!(final_state.count, 11);

        let recorded = events.lock().unwrap().clone();
        assert_eq!(
            recorded,
            vec![
                HookEvent::AgentStart("step_1".to_string(), 10),
                HookEvent::NodeStart("step_1".to_string(), 10),
                HookEvent::StateChange("step_1".to_string(), 11),
                HookEvent::NodeEnd("step_1".to_string(), 11, true),
                HookEvent::ConditionalStart("step_1".to_string(), 11),
                HookEvent::ConditionalEnd("step_1".to_string(), "finish".to_string(), 11),
                HookEvent::NodeStart("finish".to_string(), 11),
                HookEvent::NodeEnd("finish".to_string(), 11, true),
                HookEvent::AgentEnd("finish".to_string(), 11),
            ]
        );
    }

    #[test]
    fn test_custom_struct_graph_hook() {
        use std::sync::{Arc, Mutex};
        use crate::hooks::GraphHook;

        #[derive(Default, Clone)]
        struct MyAuditHook {
            pub log: Arc<Mutex<Vec<String>>>,
        }

        impl GraphHook<TestState> for MyAuditHook {
            fn on_agent_start(&self, entry_point: &str, state: &TestState) {
                self.log.lock().unwrap().push(format!("start:{entry_point}:{}", state.count));
            }

            fn on_agent_state_change(&self, node_name: &str, state: &TestState) {
                self.log.lock().unwrap().push(format!("changed:{node_name}:{}", state.count));
            }

            fn on_node_end(&self, node_name: &str, state: &TestState, result: &Result<(), GraphError>) {
                self.log.lock().unwrap().push(format!("end:{node_name}:{}:{}", state.count, result.is_ok()));
            }

            fn on_agent_end(&self, finish_point: &str, state: &TestState) {
                self.log.lock().unwrap().push(format!("finish:{finish_point}:{}", state.count));
            }
        }

        let audit = MyAuditHook::default();
        let log_ref = audit.log.clone();

        let mut graph: Graph<TestState> = Graph::new();
        graph
            .add_hook(audit)
            .add_node("step_a".to_string(), increment)
            .add_node("finish".to_string(), finish)
            .add_edge("step_a".to_string(), "finish".to_string())
            .set_entry_point("step_a".to_string())
            .set_finish_point("finish".to_string());

        let compiled = graph.compile().unwrap();
        compiled.invoke(TestState { count: 0 }).unwrap();

        let history = log_ref.lock().unwrap().clone();
        assert_eq!(
            history,
            vec![
                "start:step_a:0",
                "changed:step_a:1",
                "end:step_a:1:true",
                "end:finish:1:true",
                "finish:finish:1",
            ]
        );
    }

    #[test]
    fn test_state_context_multiple_mutations_and_read_only() {
        use std::sync::{Arc, Mutex};

        let state_changes = Arc::new(Mutex::new(Vec::new()));
        let sc_clone = state_changes.clone();

        let mut graph: Graph<TestState> = Graph::new();
        graph
            .on_agent_state_change(move |node, state| {
                sc_clone.lock().unwrap().push((node.to_string(), state.count));
            })
            // Node that updates state multiple times and uses set()
            .add_node("mutator".to_string(), |ctx| {
                // Read via Deref
                assert_eq!(ctx.count, 0);

                // First mutation via update()
                ctx.update(|s| s.count += 5);

                // Second mutation via update()
                ctx.update(|s| s.count += 10);

                // Third mutation via set()
                ctx.set(TestState { count: 100 });

                Ok(())
            })
            // Read-only node that performs no state mutations
            .add_node("read_only".to_string(), |ctx| {
                assert_eq!(ctx.count, 100);
                assert_eq!(ctx.node_name(), "read_only");
                assert_eq!(ctx.get().count, 100);
                Ok(())
            })
            .add_edge("mutator".to_string(), "read_only".to_string())
            .set_entry_point("mutator".to_string())
            .set_finish_point("read_only".to_string());

        let compiled = graph.compile().unwrap();
        let final_state = compiled.invoke(TestState { count: 0 }).unwrap();
        assert_eq!(final_state.count, 100);

        // Verify state change hooks fired exactly when mutated, and NOT for read_only
        let changes = state_changes.lock().unwrap().clone();
        assert_eq!(
            changes,
            vec![
                ("mutator".to_string(), 5),
                ("mutator".to_string(), 15),
                ("mutator".to_string(), 100),
            ]
        );
    }

    #[test]
    fn test_hook_on_node_error() {
        use std::sync::{Arc, Mutex};

        let last_error = Arc::new(Mutex::new(None));
        let last_error_clone = last_error.clone();

        let mut graph: Graph<TestState> = Graph::new();
        graph
            .on_node_end(move |_node, _state, res| {
                if let Err(e) = res {
                    *last_error_clone.lock().unwrap() = Some(e.clone());
                }
            })
            .add_node("failing".to_string(), |_| {
                Err(GraphError::ExecutionError("Intentional failure".to_string()))
            })
            .set_entry_point("failing".to_string())
            .set_finish_point("failing".to_string());

        let compiled = graph.compile().unwrap();
        let res = compiled.invoke(TestState { count: 0 });
        assert!(res.is_err());

        let err = last_error.lock().unwrap().clone();
        assert_eq!(
            err,
            Some(GraphError::ExecutionError("Intentional failure".to_string()))
        );
    }
}

