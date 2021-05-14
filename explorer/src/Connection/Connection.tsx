import * as React from "react";
import MsgPack from "msgpack5";
import TetherAgent, { Input, Output } from "tether";

interface ConnectionProps {
  path: string;
  host: string;
  port: number;
}

interface ConnectionState {
  connected: boolean;
  sent: any[];
  received: any[];
  nextMessage: string;
  sender: Output | null;
}

let agent: TetherAgent = new TetherAgent("browser", "explorer");

const msgpack = MsgPack();

class Connection extends React.Component<ConnectionProps, ConnectionState> {
  constructor(props: ConnectionProps) {
    super(props);
    this.state = {
      connected: false,
      sent: [],
      received: [],
      nextMessage: `{ "hello": "world" }`,
      sender: null,
    };
  }

  async componentDidMount() {
    const { port, path, host } = this.props;

    try {
      await agent.connect({ protocol: "ws", host, port, path });
      console.info("connected!");
      const sender = await agent.createOutput("browserData");
      this.setState({ sender });

      const receiver = await agent.createInput("dummyData");
      receiver.on("message", (topic, message) => {
        const decoded = msgpack.decode(message);
        console.log("received message:", {
          topic,
          raw: message.toString(),
          decoded,
          mType: typeof decoded,
        });
        this.setState({
          received: [...this.state.received, decoded],
        });
      });
    } catch (e) {
      console.error("error connecting Tether:", e);
    }
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
                console.log("attempting to send", {
                  json,
                  mType: typeof json,
                  encodedMessage,
                  toStr: encodedMessage.toString(),
                  altStr: msgpack.encode(json).toString,
                  toBuffer: msgpack.encode(json),
                });
                const { sender } = this.state;
                if (sender) {
                  console.log("sending");
                  sender.publish(encodedMessage);
                }
              } catch (e) {
                console.log("not valid JSON:", e);
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
