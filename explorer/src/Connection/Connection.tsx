import * as React from "react";
import mqtt, { MqttClient } from "mqtt";

interface ConnectionProps {
  path: string;
  address: string;
  port: number;
}

interface ConnectionState {
  connected: boolean;
  sent: number;
  received: number;
}

let client: MqttClient | null = null;

class Connection extends React.Component<ConnectionProps, ConnectionState> {
  constructor(props: ConnectionProps) {
    super(props);
    this.state = {
      connected: false,
      sent: 0,
      received: 0,
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
          client?.publish("presence", "Hello mqtt");
          this.setState({ sent: this.state.sent + 1 });
        }
      });
    });

    client.on("message", (topic, message) => {
      // message is Buffer
      console.log(message.toString());
      this.setState({ received: this.state.received + 1 });
      // client?.end()
    });
  }

  render() {
    return (
      <div>
        <h2>RabbitMQ Connection</h2>
        <div>
          <h3>Details</h3>
          <code>{JSON.stringify(this.props, null, 2)}</code>
        </div>

        <div>
          <h3>State</h3>
          <code>{JSON.stringify(this.state, null, 2)}</code>
        </div>
      </div>
    );
  }
}

export default Connection;
