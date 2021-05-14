import * as React from "react";
import MsgPack from "msgpack5";
import TetherAgent, { Output } from "tether";

interface ConnectionProps {
  path: string;
  host: string;
  port: number;
}

interface ConnectionState {
  connected: boolean;
  receivedOnInput: any[];
  receivedAny: any[];
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
      receivedOnInput: [],
      receivedAny: [],
      nextMessage: `{ "hello": "world" }`,
      sender: null,
    };
  }

  async componentDidMount() {
    const { port, path, host } = this.props;

    try {
      // Note how we only need to override the protocol and port to
      // get MQTT via websocket (for the browser) rather than TCP
      // (for other runtimes, e.g. NodeJS)
      await agent.connect({ protocol: "ws", host, port, path });

      console.info("connected!");
      this.setState({ connected: true });

      const sender = await agent.createOutput("browserData");
      this.setState({ sender });

      const input = await agent.createInput("dummyData");

      input.on("message", (topic, message) => {
        const decoded = msgpack.decode(message);
        // console.log("INPUT message:", {
        //   topic,
        //   raw: message.toString(),
        //   decoded,
        //   mType: typeof decoded,
        // });
        console.log("INPUT message:", topic);
        this.setState({
          receivedOnInput: [
            ...this.state.receivedOnInput,
            { topic, decoded, onInput: input.getDefinition().name },
          ],
        });
      });

      const client = agent.getClient();
      client.subscribe("#");
      client.on("message", (topic, message) => {
        console.log("CLIENT message:", topic);
        const decoded = msgpack.decode(message);
        this.setState({
          receivedAny: [
            ...this.state.receivedAny,
            { topic, decoded, onInput: null },
          ],
        });
      });
    } catch (e) {
      console.error("error connecting Tether:", e);
    }
  }

  render() {
    const { connected, receivedOnInput, receivedAny } = this.state;

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
          <h3>Messages received on named Input ({receivedOnInput.length})</h3>
          <ul>
            {receivedOnInput.map((m, index) => (
              <li key={`received-${index}`}>{JSON.stringify(m)}</li>
            ))}
          </ul>
        </div>

        <div>
          <h3>Messages received on any topic ({receivedAny.length})</h3>
          <ul>
            {receivedAny.map((m, index) => (
              <li key={`received-${index}`}>{JSON.stringify(m)}</li>
            ))}
          </ul>
        </div>
      </div>
    );
  }
}

export default Connection;
