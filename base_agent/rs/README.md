# Tether Agent for Rust

## Examples

Create an agent, specify `None` for to use defaults for MQTT Broker IP address and/or Agent ID (Group): 
```
    let agent = TetherAgent::new("RustDemoAgent", None, None);
    agent.connect();
```

Create an Output Plug:
```
    let custom_output = agent.create_output_plug("custom", None, None).unwrap();

```

If your data structure can be Serialised using serde, go ahead and encode+publish in one step:
```
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CustomStruct {
    foo: String,
    bar: f32,
}
/// ...
        let custom_message = CustomStruct {
            foo: "hello".into(),
            bar: 0.42,
        };
        agent
            .encode_and_publish(&custom_output, custom_message)
            .unwrap();
```

Or create an Input Plug:
```
    let input_one = agent.create_input_plug("one", None, None).unwrap();
```

And check for messages synchronously:
```
if let Some((plug_name, message)) = agent.check_messages() {
            if &input_one.name == plug_name.as_str() {
               // Always check against the plug name(s) you're looking for!
```

## Approach

This "Base Agent" implementation assumes that the client (your application) will retain ownership of any Input and Output Plugs, as well as the instance of the TetherAgent struct.

This means that the TetherAgent retains no "memory" of any Input or Output Plugs that you have created. 
Therefore, you must keep your own individual variables which reference the Plugs you have created, or store them in a `Vec<&PlugDefinition>` as necessary.

## Publishing
The following functions can be called on the `TetherAgent` instance:

- `publish`: expects an already-encoded Vector slice of u8 (i.e. a buffer)
- `encode_and_publish`: can automatically encode any data type or struct to a valid message as long as the `data` implements the Serde `Serialize` trait

In both cases, you provide a pointer to the `PlugDefinition` so that the Agent can publish on the appropriate topic with the correct QOS for the plug.

## Subscribing
The `create_input_plug` function has a side effect: the client subscription.

For now, checking messages is done synchronously. The same function should be called as often as possible (e.g. once per frame or on a timed thread, etc.) on the `TetherAgent` instance:

- `check_messages`

Note that in the case of subscribing (Input Plugs) you do not need to pass the plug definition. This means that **you** need to check any returned messages against the plug name(s) you want to match against for your Input Plug(s).

This is why `check_messages` returns Some(String, Message) where the String is the plug name - this will be parsed automatically from the message topic.
