use crate::pregel::{
    BinaryOperatorAggregate, ChannelValues, EphemeralValue, LastValue, NodeBuilder, Pregel,
    PregelError, Topic,
};

#[test]
fn test_pregel_single_node() {
    // Matches LangGraph Example: Single node application
    // node1 = NodeBuilder().subscribe_only("a").do(lambda x: x + x).write_to("b")
    let node1 = NodeBuilder::new("node1")
        .subscribe_only("a")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("b");

    let mut app = Pregel::new();
    app.add_channel("a", EphemeralValue::<String>::new());
    app.add_channel("b", EphemeralValue::<String>::new());
    app.add_node(node1);
    app.set_input_channels(vec!["a"]);
    app.set_output_channels(vec!["b"]);

    let mut inputs = ChannelValues::new();
    inputs.insert("a", "foo".to_string());

    let output = app.invoke_values(inputs).unwrap();
    let b: String = output.get("b").expect("channel b should have value");
    assert_eq!(b, "foofoo");
}

#[test]
fn test_pregel_multiple_nodes_and_output_channels() {
    // Matches LangGraph Example: Using multiple nodes and multiple output channels
    let node1 = NodeBuilder::new("node1")
        .subscribe_only("a")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("b");

    let node2 = NodeBuilder::new("node2")
        .subscribe_only("b")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("c");

    let mut app = Pregel::new();
    app.add_channel("a", EphemeralValue::<String>::new());
    app.add_channel("b", LastValue::<String>::new());
    app.add_channel("c", EphemeralValue::<String>::new());
    app.add_node(node1);
    app.add_node(node2);
    app.set_input_channels(vec!["a"]);
    app.set_output_channels(vec!["b", "c"]);

    let mut inputs = ChannelValues::new();
    inputs.insert("a", "foo".to_string());

    let output = app.invoke_values(inputs).unwrap();
    let b: String = output.get("b").expect("channel b");
    let c: String = output.get("c").expect("channel c");

    assert_eq!(b, "foofoo");
    assert_eq!(c, "foofoofoofoo");
}

#[test]
fn test_pregel_topic_channel_accumulation() {
    // Matches LangGraph Example: Using a Topic channel
    let node1 = NodeBuilder::new("node1")
        .subscribe_only("a")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("b")
        .write_to("c");

    let node2 = NodeBuilder::new("node2")
        .subscribe_only("b")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("c");

    let mut app = Pregel::new();
    app.add_channel("a", EphemeralValue::<String>::new());
    app.add_channel("b", EphemeralValue::<String>::new());
    app.add_channel("c", Topic::<String>::new(true)); // accumulate = true
    app.add_node(node1);
    app.add_node(node2);
    app.set_input_channels(vec!["a"]);
    app.set_output_channels(vec!["c"]);

    let mut inputs = ChannelValues::new();
    inputs.insert("a", "foo".to_string());

    let output = app.invoke_values(inputs).unwrap();
    let c: Vec<String> = output.get("c").expect("channel c");

    assert_eq!(c, vec!["foofoo", "foofoofoofoo"]);
}

#[test]
fn test_pregel_binary_operator_aggregate() {
    // Matches LangGraph Example: Using a BinaryOperatorAggregate channel
    let node1 = NodeBuilder::new("node1")
        .subscribe_only("a")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("b")
        .write_to("c");

    let node2 = NodeBuilder::new("node2")
        .subscribe_only("b")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("c");

    let reducer = |current: Option<String>, update: String| match current {
        Some(cur) => format!("{cur} | {update}"),
        None => update,
    };

    let mut app = Pregel::new();
    app.add_channel("a", EphemeralValue::<String>::new());
    app.add_channel("b", EphemeralValue::<String>::new());
    app.add_channel(
        "c",
        BinaryOperatorAggregate::<String>::new(reducer),
    );
    app.add_node(node1);
    app.add_node(node2);
    app.set_input_channels(vec!["a"]);
    app.set_output_channels(vec!["c"]);

    let mut inputs = ChannelValues::new();
    inputs.insert("a", "foo".to_string());

    let output = app.invoke_values(inputs).unwrap();
    let c: String = output.get("c").expect("channel c");

    assert_eq!(c, "foofoo | foofoofoofoo");
}

#[test]
fn test_pregel_cycle_with_threshold() {
    // Matches LangGraph Example: Introducing a cycle
    // Loop until string length >= 10, then return None
    let example_node = NodeBuilder::new("example_node")
        .subscribe_only("value")
        .do_pure_option(|x: String| {
            if x.len() < 10 {
                Some(format!("{x}{x}"))
            } else {
                None
            }
        })
        .write_to("value");

    let mut app = Pregel::new();
    app.add_channel("value", EphemeralValue::<String>::new());
    app.add_node(example_node);
    app.set_input_channels(vec!["value"]);
    app.set_output_channels(vec!["value"]);

    let mut inputs = ChannelValues::new();
    inputs.insert("value", "a".to_string());

    // Steps:
    // "a" (len 1 < 10) -> "aa"
    // "aa" (len 2 < 10) -> "aaaa"
    // "aaaa" (len 4 < 10) -> "aaaaaaaa"
    // "aaaaaaaa" (len 8 < 10) -> "aaaaaaaaaaaaaaaa" (len 16)
    // "aaaaaaaaaaaaaaaa" (len 16 >= 10) -> None (no write, halts)
    let output = app.invoke_values(inputs).unwrap();
    let val: String = output.get("value").expect("value channel");
    assert_eq!(val, "aaaaaaaaaaaaaaaa");
}

#[test]
fn test_pregel_bsp_isolation() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Both node1 and node2 trigger in the same superstep from channel "start".
    // node1 writes to "ch1".
    // node2 checks if "ch1" is in snapshot during execution. Because of BSP isolation,
    // node1's write to "ch1" must NOT be visible to node2 during this step.
    let saw_ch1_prematurely = Arc::new(AtomicBool::new(false));
    let saw_ref = saw_ch1_prematurely.clone();

    let node1 = NodeBuilder::new("node1")
        .subscribe_only("start")
        .do_pure(|_: i32| "data_from_node1".to_string())
        .write_to("ch1");

    let node2 = NodeBuilder::new("node2")
        .subscribe_only("start")
        .do_values_pure(move |values| {
            if values.contains_key("ch1") {
                saw_ref.store(true, Ordering::SeqCst);
            }
            vec![]
        });

    let mut app = Pregel::new();
    app.add_channel("start", EphemeralValue::<i32>::new());
    app.add_channel("ch1", EphemeralValue::<String>::new());
    app.add_node(node1);
    app.add_node(node2);
    app.set_input_channels(vec!["start"]);
    app.set_output_channels(vec!["ch1"]);

    let mut inputs = ChannelValues::new();
    inputs.insert("start", 42);

    let output = app.invoke_values(inputs).unwrap();
    assert!(!saw_ch1_prematurely.load(Ordering::SeqCst));
    assert_eq!(output.get::<String>("ch1").unwrap(), "data_from_node1");
}

#[test]
fn test_pregel_max_steps_exceeded() {
    // Infinite cycle without stop condition should hit max_steps
    let loop_node = NodeBuilder::new("loop_node")
        .subscribe_only("count")
        .do_pure(|x: i32| x + 1)
        .write_to("count");

    let mut app = Pregel::new();
    app.add_channel("count", EphemeralValue::<i32>::new());
    app.add_node(loop_node);
    app.set_input_channels(vec!["count"]);
    app.set_max_steps(5);

    let mut inputs = ChannelValues::new();
    inputs.insert("count", 0);

    let result = app.invoke_values(inputs);
    assert_eq!(result.unwrap_err(), PregelError::MaxStepsExceeded(5));
}

#[test]
fn test_pregel_invoke_single_convenience() {
    let node = NodeBuilder::new("double")
        .subscribe_only("num")
        .do_pure(|x: i32| x * 2)
        .write_to("out");

    let mut app = Pregel::new();
    app.add_channel("num", EphemeralValue::<i32>::new());
    app.add_channel("out", EphemeralValue::<i32>::new());
    app.add_node(node);
    app.set_input_channels(vec!["num"]);
    app.set_output_channels(vec!["out"]);

    let res: i32 = app.invoke_single("num", 21, "out").unwrap();
    assert_eq!(res, 42);
}
