import React, { ChangeEvent } from "react";
import { encode, decode } from "@msgpack/msgpack";
import { Slider } from '@material-ui/core';
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
  sliderValue: number;
}

let agent: TetherAgent = new TetherAgent("browser", "explorer");

class Connection extends React.Component<ConnectionProps, ConnectionState> {
  constructor(props: ConnectionProps) {
    super(props);
    this.state = {
      connected: false,
      receivedOnInput: [],
      receivedAny: [],
      nextMessage: `{ "hello": "world" }`,
      sender: null,
      sliderValue: 0,
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
        const decoded = decode(message);
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
        const decoded = decode(message);
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

  handleChangeSlider = (event: ChangeEvent<{}>, newValue: number | number[]) => {
    const value = Array.isArray(newValue) ? newValue[0] : newValue;
    if (value !== this.state.sliderValue) {
      this.setState({ sliderValue: value });
      this.sendMessage({ slider: value }, this.state.sender);
    }
  };

  sendMessage = (message: object, output: Output | null) => {
    if (!output) {
      console.error("Cannot send on null output");
      return;
    }
    try {
      const encodedMessage = Buffer.from(encode(message));
      console.log("attempting to send", {
        message,
        mType: typeof message,
        encodedMessage,
        toStr: encodedMessage.toString(),
        altStr: encode(message).toString,
        toBuffer: encode(message),
      });
      const { sender } = this.state;
      if (sender) {
        console.log("sending");
        sender.publish(encodedMessage);
      }
    } catch (e) {
      console.log("not valid JSON:", e);
    }
  }

  render() {
    const { connected, receivedOnInput, receivedAny, sliderValue } = this.state;

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
              const json = JSON.parse(this.state.nextMessage);
              this.sendMessage(json, this.state.sender);
            }}
          >
            send
          </button>
        </div>
        <div>
          <h4>Slide to send</h4>
          <Slider
            value={sliderValue}
            min={0}
            max={255}
            onChange={this.handleChangeSlider}
            style={{ width: 200 , marginLeft: "1em"}}
          />
          <p style={{ margin: "0 0 0 1em" }}>
            {`{ slider: ${sliderValue} }`}
          </p>
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
