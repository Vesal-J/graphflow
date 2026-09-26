use graphflow::{init_logger, Graph};

#[derive(Debug)]
struct AgentState {
    pub count: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize styled logger
    init_logger();

    log::info!("Building Counter Loop Graph...");
    let mut graph: Graph<AgentState> = Graph::new();

    graph
        .add_node("increment".to_string(), |state| {
            log::info!("Node 'increment': count was {}, incrementing by 1", state.count);
            state.count += 1;
            Ok(())
        })
        .add_node("double".to_string(), |state| {
            log::info!("Node 'double': count was {}, doubling", state.count);
            state.count *= 2;
            Ok(())
        })
        .add_node("finish".to_string(), |state| {
            log::info!("Node 'finish': target reached with final count {}", state.count);
            Ok(())
        })
        .add_edge("increment".to_string(), "double".to_string())
        .add_conditional_edge("double".to_string(), |state| {
            log::debug!("Evaluating conditional edge at count {}", state.count);
            if state.count < 10 {
                "increment".to_string()
            } else {
                "finish".to_string()
            }
        })
        .set_entry_point("increment".to_string())
        .set_finish_point("finish".to_string());

    let compiled = graph.compile()?;

    let initial_state = AgentState { count: 1 };
    log::info!("Starting graph invocation with count={}", initial_state.count);
    let final_state = compiled.invoke(initial_state)?;
    log::info!("Graph execution complete! Final count: {}", final_state.count);

    Ok(())
}
