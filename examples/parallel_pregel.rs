use graphflow::Graph;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct ParallelState {
    query: String,
    results: Arc<Mutex<Vec<String>>>,
}

fn search_web(state: &mut ParallelState) -> Result<(), std::fmt::Error> {
    println!("[Thread {:?}] Searching web for '{}'...", thread::current().id(), state.query);
    thread::sleep(Duration::from_millis(50));
    state.results.lock().unwrap().push("Web Result: Rust 2024 is awesome".to_string());
    Ok(())
}

fn search_database(state: &mut ParallelState) -> Result<(), std::fmt::Error> {
    println!("[Thread {:?}] Querying database for '{}'...", thread::current().id(), state.query);
    thread::sleep(Duration::from_millis(50));
    state.results.lock().unwrap().push("DB Result: 42 records found".to_string());
    Ok(())
}

fn search_docs(state: &mut ParallelState) -> Result<(), std::fmt::Error> {
    println!("[Thread {:?}] Scanning docs for '{}'...", thread::current().id(), state.query);
    thread::sleep(Duration::from_millis(50));
    state.results.lock().unwrap().push("Doc Result: LangGraph Pregel architecture".to_string());
    Ok(())
}

fn aggregate_results(state: &mut ParallelState) -> Result<(), std::fmt::Error> {
    let collected = state.results.lock().unwrap().clone();
    println!("[Thread {:?}] Join barrier reached! Aggregating {} results:", thread::current().id(), collected.len());
    for (i, res) in collected.iter().enumerate() {
        println!("  {}. {}", i + 1, res);
    }
    Ok(())
}

fn finish(_state: &mut ParallelState) -> Result<(), std::fmt::Error> {
    println!("Workflow complete!");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph: Graph<ParallelState> = Graph::new();

    graph
        .add_node("start".to_string(), |_| {
            println!("Starting parallel execution fan-out...");
            Ok(())
        })
        .add_node("search_web".to_string(), search_web)
        .add_node("search_db".to_string(), search_database)
        .add_node("search_docs".to_string(), search_docs)
        .add_node("aggregate".to_string(), aggregate_results)
        .add_node("finish".to_string(), finish)
        // Fan-out: start triggers all 3 search tasks in parallel in the next Pregel superstep
        .add_parallel_edge(
            "start".to_string(),
            vec![
                "search_web".to_string(),
                "search_db".to_string(),
                "search_docs".to_string(),
            ],
        )
        // Fan-in: all 3 workers converge to aggregate
        .add_edge("search_web".to_string(), "aggregate".to_string())
        .add_edge("search_db".to_string(), "aggregate".to_string())
        .add_edge("search_docs".to_string(), "aggregate".to_string())
        .add_edge("aggregate".to_string(), "finish".to_string())
        .set_entry_point("start".to_string())
        .set_finish_point("finish".to_string());

    let compiled = graph.compile().map_err(|e| format!("Graph compile failed: {e}"))?;

    let state = ParallelState {
        query: "Pregel workflow".to_string(),
        results: Arc::new(Mutex::new(Vec::new())),
    };

    println!("Invoking graph with Pregel execution model:");
    compiled.invoke(state)?;

    Ok(())
}
