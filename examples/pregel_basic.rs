use graphflow::init_logger;
use graphflow::pregel::{
    BinaryOperatorAggregate, ChannelValues, EphemeralValue, LastValue, NodeBuilder, Pregel, Topic,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logger();
    log::info!("=== LangGraph-style Pregel BSP Engine in Rust ===");

    // -----------------------------------------------------------------------
    // Example 1: Single Node Application
    // -----------------------------------------------------------------------
    log::info!("--- Example 1: Single Node Application ---");
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
    let out = app.invoke_values(inputs)?;
    log::info!("Output: {{'b': '{}'}}", out.get::<String>("b").unwrap());

    // -----------------------------------------------------------------------
    // Example 2: Multiple Nodes & Multiple Output Channels
    // -----------------------------------------------------------------------
    log::info!("--- Example 2: Multiple Nodes & Output Channels ---");
    let n1 = NodeBuilder::new("node1")
        .subscribe_only("a")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("b");

    let n2 = NodeBuilder::new("node2")
        .subscribe_only("b")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("c");

    let mut app2 = Pregel::new();
    app2.add_channel("a", EphemeralValue::<String>::new());
    app2.add_channel("b", LastValue::<String>::new());
    app2.add_channel("c", EphemeralValue::<String>::new());
    app2.add_node(n1);
    app2.add_node(n2);
    app2.set_input_channels(vec!["a"]);
    app2.set_output_channels(vec!["b", "c"]);

    let mut inputs2 = ChannelValues::new();
    inputs2.insert("a", "foo".to_string());
    let out2 = app2.invoke_values(inputs2)?;
    log::info!(
        "Output: {{'b': '{}', 'c': '{}'}}",
        out2.get::<String>("b").unwrap(),
        out2.get::<String>("c").unwrap()
    );

    // -----------------------------------------------------------------------
    // Example 3: Topic Channel with Accumulation
    // -----------------------------------------------------------------------
    log::info!("--- Example 3: Topic PubSub Channel ---");
    let t_node1 = NodeBuilder::new("node1")
        .subscribe_only("a")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("b")
        .write_to("c");

    let t_node2 = NodeBuilder::new("node2")
        .subscribe_only("b")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("c");

    let mut app3 = Pregel::new();
    app3.add_channel("a", EphemeralValue::<String>::new());
    app3.add_channel("b", EphemeralValue::<String>::new());
    app3.add_channel("c", Topic::<String>::new(true)); // accumulate = true
    app3.add_node(t_node1);
    app3.add_node(t_node2);
    app3.set_input_channels(vec!["a"]);
    app3.set_output_channels(vec!["c"]);

    let mut inputs3 = ChannelValues::new();
    inputs3.insert("a", "foo".to_string());
    let out3 = app3.invoke_values(inputs3)?;
    log::info!("Output: {{'c': {:?}}}", out3.get::<Vec<String>>("c").unwrap());

    // -----------------------------------------------------------------------
    // Example 4: BinaryOperatorAggregate with Reducer
    // -----------------------------------------------------------------------
    log::info!("--- Example 4: BinaryOperatorAggregate Reducer ---");
    let r_node1 = NodeBuilder::new("node1")
        .subscribe_only("a")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("b")
        .write_to("c");

    let r_node2 = NodeBuilder::new("node2")
        .subscribe_only("b")
        .do_pure(|x: String| format!("{x}{x}"))
        .write_to("c");

    let reducer = |current: Option<String>, update: String| match current {
        Some(cur) => format!("{cur} | {update}"),
        None => update,
    };

    let mut app4 = Pregel::new();
    app4.add_channel("a", EphemeralValue::<String>::new());
    app4.add_channel("b", EphemeralValue::<String>::new());
    app4.add_channel("c", BinaryOperatorAggregate::<String>::new(reducer));
    app4.add_node(r_node1);
    app4.add_node(r_node2);
    app4.set_input_channels(vec!["a"]);
    app4.set_output_channels(vec!["c"]);

    let mut inputs4 = ChannelValues::new();
    inputs4.insert("a", "foo".to_string());
    let out4 = app4.invoke_values(inputs4)?;
    log::info!("Output: {{'c': '{}'}}", out4.get::<String>("c").unwrap());

    // -----------------------------------------------------------------------
    // Example 5: Cycle with Termination
    // -----------------------------------------------------------------------
    log::info!("--- Example 5: Cycle with Threshold ---");
    let cycle_node = NodeBuilder::new("example_node")
        .subscribe_only("value")
        .do_pure_option(|x: String| {
            if x.len() < 10 {
                Some(format!("{x}{x}"))
            } else {
                None
            }
        })
        .write_to("value");

    let mut app5 = Pregel::new();
    app5.add_channel("value", EphemeralValue::<String>::new());
    app5.add_node(cycle_node);
    app5.set_input_channels(vec!["value"]);
    app5.set_output_channels(vec!["value"]);

    let mut inputs5 = ChannelValues::new();
    inputs5.insert("value", "a".to_string());
    let out5 = app5.invoke_values(inputs5)?;
    log::info!("Output: {{'value': '{}'}}", out5.get::<String>("value").unwrap());

    log::info!("All Pregel examples completed successfully!");
    Ok(())
}
