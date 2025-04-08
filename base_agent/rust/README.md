# Tether Agent for Rust

## Create and Connect

Build a Tether Agent using defaults, and connect it automatically:

```
 let mut agent = TetherAgentOptionsBuilder::new("RustDemoAgent")
        .build()
        .expect("failed to connect Tether");
```

## Send Messages
Create a Channel Sender, passing in a ref to the Tether Agent you created:

```
    let sender_definition = PlugOptionsBuilder::create_sender("customValues")
        .build(&mut agent)
        .expect("failed to create sender");

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
            .encode_and_publish(&sender_definition, custom_message)
            .unwrap();
```

Or encode first and use `agent.send`

## Receive Messages
Create a Channel Receiver:
```
    let receiver_defintion = PlugOptionsBuilder::create_input("customValues")
        .build(&mut agent)
        .expect("failed to create receiver");
```

And check for messages synchronously:

```
if let Some((topic, payload)) = agent.check_messages() {
  // Always check against the Channel this message "belongs" to!
  if receiver_defintion.matches(&topic) {
      // Decode the message payload, if applicable
      let decoded = rmp_serde::from_slice::<CustomStruct>::(&payload);
      ...
  }
```
