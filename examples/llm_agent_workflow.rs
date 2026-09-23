use rustchain::Graph;

#[derive(Debug)]
struct AgentState {
    query: String,
    iterations: usize,
    max_iterations: usize,
    has_sufficient_context: bool,
    response: Option<String>,
}

fn plan(state: &mut AgentState) -> Result<(), std::fmt::Error> {
    println!("[Plan] Planning response for query: '{}'", state.query);
    state.iterations += 1;
    Ok(())
}

fn search_tools(state: &mut AgentState) -> Result<(), std::fmt::Error> {
    state.iterations += 1;
    println!("[Tool] Querying search tool (attempt {})...", state.iterations);
    if state.iterations >= 2 {
        state.has_sufficient_context = true;
    }
    Ok(())
}

fn route_decision(state: &AgentState) -> String {
    if state.has_sufficient_context || state.iterations >= state.max_iterations {
        "synthesize".to_string()
    } else {
        "search".to_string()
    }
}

fn synthesize(state: &mut AgentState) -> Result<(), std::fmt::Error> {
    println!("[Synthesize] Generating final response...");
    state.response = Some(format!("Final answer for '{}'", state.query));
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph: Graph<AgentState> = Graph::new();

    graph
        .add_node("plan".to_string(), plan)
        .add_node("search".to_string(), search_tools)
        .add_node("synthesize".to_string(), synthesize)
        .add_edge("plan".to_string(), "search".to_string())
        .add_conditional_edge("search".to_string(), route_decision)
        .set_entry_point("plan".to_string())
        .set_finish_point("synthesize".to_string());

    let compiled_graph = graph
        .compile()
        .map_err(|e| format!("Graph compilation error: {e}"))?;

    let state = AgentState {
        query: "What is the capital of Rustland?".to_string(),
        iterations: 0,
        max_iterations: 3,
        has_sufficient_context: false,
        response: None,
    };

    compiled_graph.invoke(state)?;

    println!("Agent workflow finished successfully.");
    Ok(())
}
