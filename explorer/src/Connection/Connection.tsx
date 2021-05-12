import * as React from "react";
import mqtt, { MqttClient } from "mqtt";

interface ConnectionProps {
  path: string;
  address: string;
  port: number;
}

interface ConnectionState {
  connected: boolean;
  sent: Buffer[];
  received: Buffer[];
}

let client: MqttClient | null = null;

class Connection extends React.Component<ConnectionProps, ConnectionState> {
  constructor(props: ConnectionProps) {
    super(props);
    this.state = {
      connected: false,
      sent: [],
      received: [],
    };
  }

  componentDidMount() {
    const { address, port, path } = this.props;
    client = mqtt.connect(`${address}:${port}${path}`);

    client.on("connect", () => {
      console.log("connected!");
      this.setState({ connected: true });
      client?.subscribe("presence", (err) => {
        if (!err) {
          const dummyMessage = "Hello mqtt";
          client?.publish("presence", dummyMessage);
          const msgBuffer = Buffer.from(dummyMessage);
          this.setState({ sent: [...this.state.sent, msgBuffer] });
        }
      });
    });

    client.on("message", (topic, message) => {
      // message is Buffer
      console.log({ message });
      this.setState({ received: [...this.state.received, message] });
      // client?.end()
    });
  }

  render() {
    const { connected, sent, received } = this.state;

    return (
      <div>
        <h2>RabbitMQ Connection</h2>
        <div>
          <h3>Details</h3>
          <code>{JSON.stringify(this.props, null, 2)}</code>
        </div>

        <div>
          <h3>Stats</h3>
          <code>
            {JSON.stringify(
              {
                connected,
                sentCount: sent.length,
                receivedCount: received.length,
              },
              null,
              2
            )}
          </code>
        </div>

        <div>
          <h3>Messages received</h3>
          <ul>
            {this.state.received.map((m, index) => (
              <li key={`received-${index}`}>{m.toString()}</li>
            ))}
          </ul>
        </div>
      </div>
    );
  }
}

export default Connection;
