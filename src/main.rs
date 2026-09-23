use rustchain::Graph;

struct AgentState {
    count: usize,
}

fn main() {
    let mut graph: Graph<AgentState> = Graph::new();

    graph
        .add_node("increment".to_string(), |state| {
            println!("adding 1 to state count");

            state.count += 1;

            Ok(())
        })
        .add_node("increment2".to_string(), |state| {
            println!("adding 2 to state count");

            state.count += 2;

            Ok(())
        })
        .add_node("finish".to_string(), |_state| {
            println!("finishing the graph");

            Ok(())
        })
        .add_conditional_edge("increment2".to_string(), |state| {
            println!("conditional, current count: {}", state.count);
            if state.count < 5 {
                String::from("increment")
            } else {
                String::from("finish")
            }
        })
        .add_edge("increment".to_string(), "increment2".to_string())
        .add_edge("increment2".to_string(), "finish".to_string())
        .set_entry_point("increment".to_string())
        .set_finish_point("finish".to_string());

    let graph = graph.compile().unwrap();

    let state = AgentState { count: 0 };

    graph.invoke(state).unwrap();
}
