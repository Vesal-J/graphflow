use graphflow::Graph;

#[derive(Debug)]
struct AgentState {
    pub count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph: Graph<AgentState> = Graph::new();

    graph
        .add_node("increment".to_string(), |state| {
            println!("Node 'increment': count was {}, incrementing by 1", state.count);
            state.count += 1;
            Ok(())
        })
        .add_node("double".to_string(), |state| {
            println!("Node 'double': count was {}, doubling", state.count);
            state.count *= 2;
            Ok(())
        })
        .add_node("finish".to_string(), |state| {
            println!("Node 'finish': target reached with final count {}", state.count);
            Ok(())
        })
        .add_edge("increment".to_string(), "double".to_string())
        .add_conditional_edge("double".to_string(), |state| {
            println!("Evaluating conditional edge at count {}", state.count);
            if state.count < 10 {
                "increment".to_string()
            } else {
                "finish".to_string()
            }
        })
        .set_entry_point("increment".to_string())
        .set_finish_point("finish".to_string());

    let compiled = graph.compile().map_err(|e| format!("Graph validation failed: {e}"))?;

    let initial_state = AgentState { count: 1 };
    println!("Starting graph invocation...");
    compiled.invoke(initial_state)?;
    println!("Graph execution complete!");

    Ok(())
}
