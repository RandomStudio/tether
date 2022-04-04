use futures::executor::block_on;

fn main() {
  tether::dummy();

  if let Err(_err) = block_on(async {

    let agent = tether::create_agent("rustAgent".to_string(), "dummy".to_string());

    println!("agent ID: {}", agent.get_id());
    agent.connect().await;

    

    // Ok::<(), mqtt::Error>(())
    Ok::<(),()>(()) // TODO: proper error handling?
  }) {
    eprintln!("Something went wrong!");
  }
}