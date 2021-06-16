import mqtt, { AsyncMqttClient } from "async-mqtt";
import { EventEmitter } from "events";
import defaults from "./defaults";
import { v4 as uuidv4 } from "uuid";

export interface PlugDefinition {
  name: string;
  topic: string;
  flowDirection: "in" | "out";
}

class Plug extends EventEmitter {
  protected definition: PlugDefinition;
  protected client: AsyncMqttClient | null;

  constructor(client: AsyncMqttClient, definition: PlugDefinition) {
    super();
    this.client = client;
    this.definition = definition;
  }

  public getDefinition = () => this.definition;
}
export class Input extends Plug {
  subscribe = async () => {
    if (this.client === null) {
      console.warn(
        "subscribing to topic before client is connected; this is allowed but you won't receive any messages until connected"
      );
    }
    await this.client.subscribe(this.definition.topic);
    console.log("subscribed to topic", this.definition.topic);
  };
}

export class Output extends Plug {
  publish = async (content: Buffer) => {
    if (this.client === null) {
      console.error(
        "trying to send without connection; not possible until connected"
      );
    } else {
      this.client.publish(this.definition.topic, content);
    }
  };
}

export class TetherAgent {
  private agentType: string = null;
  private agentID: string = null;

  private client: AsyncMqttClient | null;

  private inputs: Input[] = [];
  private outputs: Output[] = [];

  constructor(agentType: string, agentID?: string) {
    this.agentType = agentType;
    this.agentID = agentID || uuidv4();
    this.client = null;
    console.log("Tether Agent instance:", { agentType, agentId: this.agentID });
  }

  public connect = async (overrides?: {
    protocol?: string;
    host?: string;
    port?: number;
    path?: string;
    username?: string;
    password?: string;
  }) => {
    const protocol = overrides?.protocol || defaults.broker.protocol;
    const host = overrides?.host || defaults.broker.host;
    const port = overrides?.port || defaults.broker.port;
    const path = overrides?.path || defaults.broker.path;
    const username = overrides?.username || defaults.broker.username;
    const password = overrides?.password || defaults.broker.password;

    const url = `${protocol}://${host}:${port}${path}`;

    console.log("Connecting to MQTT broker @", url);

    try {
      this.client = await mqtt.connectAsync(url, { username, password });
      console.info("Connected OK");
      this.listenForIncoming();
    } catch (error) {
      console.error("Error connecting to MQTT broker:", { error, url });
      throw error;
    }
  };

  public disconnect = async () => {
    if (this.client) {
      await this.client.end();
      console.log("MQTT client closed normally");
    } else {
      console.warn("MQTT client not available on disconnect request");
    }
  };

  /**
   * End users can get the underlying client if they like. This allows you to bypass
   * the "plugs" altogether and subscribe or publish on topics directly.
   *
   * @returns The AsyncMqttClient client instance, or null if not (yet) connected
   */
  public getClient = () => this.client;

  public getIsConnected = () => this.client !== null;

  public getInput = (name: string) =>
    this.inputs.find((p) => p.getDefinition().name === name);

  public getOutput = (name: string) =>
    this.inputs.find((p) => p.getDefinition().name === name);

  /**
   * Define a named Output to indicate some data that this agent is expected to produce/send.
   *
   * For convenience, the topic is generated once and used for every message on this Output instance when calling its `publish` function.
   */
  public createOutput = async (name: string) => {
    const definition: PlugDefinition = {
      name,
      topic: `${this.agentType}/${this.agentID}/${name}`,
      flowDirection: "out",
    };

    const output = new Output(this.client, definition);
    this.outputs.push(output);

    return output;
  };

  /**
   * Define a named Input to indicate some data this agent is expected to consume/receive.
   *
   * For convenience, the topic is assumed to end in the given `name`, e.g. an Input named "someTopic" will match messages on topics `foo/bar/someTopic` as well as `something/else/someTopic`.
   *
   * Returns an Output instance which is an EventEmitter. Events named "message" with contents (topic, message) will be emitted on this instance, but _only_ if they match the Output name.
   */
  public createInput = async (name: string) => {
    // Create a new Input
    const definition: PlugDefinition = {
      name,
      topic: `+/+/${name}`,
      flowDirection: "in",
    };
    const input = new Input(this.client, definition);

    input.subscribe();

    this.inputs.push(input);

    return input;
  };

  private listenForIncoming = () => {
    this.client.on("message", (topic, payload) => {
      const matchingInputPlug = this.inputs.find((p) =>
        topicHasPlugName(topic, p.getDefinition().name)
      );
      if (matchingInputPlug) {
        matchingInputPlug.emit("message", topic, payload);
      } else {
        console.log("message received but cannot match to Input Plug:", {
          topic,
          payload,
        });
      }
    });
  };
}

const topicHasPlugName = (topic: string, plugName: string) =>
  topic.split(`/`)[2] === plugName;

export default TetherAgent;
