import * as React from "react";
import MsgPack from "msgpack5";
import mqtt, { MqttClient } from "mqtt";

interface ConnectionProps {
  path: string;
  address: string;
  port: number;
}

interface ConnectionState {
  connected: boolean;
  sent: any[];
  received: any[];
  nextMessage: string;
}

let client: MqttClient | null = null;

const msgpack = MsgPack();

class Connection extends React.Component<ConnectionProps, ConnectionState> {
  constructor(props: ConnectionProps) {
    super(props);
    this.state = {
      connected: false,
      sent: [],
      received: [],
      nextMessage: `{ "hello": "world" }`,
    };
  }

  componentDidMount() {
    const { address, port, path } = this.props;
    client = mqtt.connect(`${address}:${port}${path}`);

    client.on("connect", () => {
      console.log("connected!");
      this.setState({ connected: true });
      client?.subscribe("#", (err) => {
        if (!err) {
          console.info("subscribed to all topics OK");
        } else {
          console.error("error subscribing", err);
        }
      });
    });

    client.on("message", (topic, message) => {
      // message is Buffer
      const decoded = msgpack.decode(message);
      console.log("received message:", {
        raw: message.toString(),
        decoded,
        mType: typeof decoded,
      });
      this.setState({
        received: [...this.state.received, decoded],
      });
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
          <h3>Send</h3>
          <input
            type="text"
            value={this.state.nextMessage}
            onChange={(event) => {
              this.setState({ nextMessage: event.target.value });
            }}
          ></input>
          <button
            onClick={() => {
              try {
                const json = JSON.parse(this.state.nextMessage);
                const encodedMessage = Buffer.from(msgpack.encode(json));
                console.log("sending", {
                  json,
                  mType: typeof json,
                  encodedMessage,
                  toStr: encodedMessage.toString(),
                  altStr: msgpack.encode(json).toString,
                  toBuffer: msgpack.encode(json),
                });
                client?.publish(
                  "dummy.browser.DummyData",
                  encodedMessage,
                  () => {
                    this.setState({ nextMessage: "" });
                  }
                );
              } catch (e) {
                console.log("not valid JSONg:", e);
              }
            }}
          >
            send
          </button>
        </div>

        <div>
          <h3>Messages received</h3>
          <ul>
            {this.state.received.map((m, index) => (
              <li key={`received-${index}`}>{JSON.stringify(m)}</li>
            ))}
          </ul>
        </div>
      </div>
    );
  }
}

export default Connection;
