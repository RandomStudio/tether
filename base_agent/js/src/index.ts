import mqtt, { AsyncMqttClient } from "async-mqtt";
import { EventEmitter } from "events";
import defaults from "./defaults";

export interface PlugDefinition {
  name: string;
  routingKey: string;
  flowDirection: "in" | "out";
}

export interface AgentInfo {
  agentType: string;
  agentID: string;
  inputs: PlugDefinition[];
  outputs: PlugDefinition[];
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
    await this.client.subscribe(this.definition.routingKey);
    console.log("subscribed to topic", this.definition.routingKey);
  };
}

export class Output extends Plug {
  publish = async (content: Buffer) => {
    if (this.client === null) {
      console.error("trying to send without connection!");
    } else {
      this.client.publish(this.definition.routingKey, content);
    }
  };
}

export class TetherAgent {
  private agentType: string = null;
  private agentID: string = null;

  private client: AsyncMqttClient | null;

  private inputs: Input[] = [];
  private outputs: Output[] = [];

  constructor(agentType: string, agentID: string) {
    this.agentType = agentType;
    this.agentID = agentID;

    this.client = null;
  }

  public connect = async (
    overrideProtocol?: string,
    overrideAddress?: string,
    overridePort?: number
  ) => {
    const protocol = overrideProtocol || defaults.broker.protocol;
    const address = overrideAddress || defaults.broker.host;
    const port = overridePort || defaults.broker.port;

    const url = `${protocol}://${address}:${port}`;

    console.log("Connecting to MQTT broker @", url);

    try {
      this.client = await mqtt.connectAsync(url);
      console.info("Connected OK");
      this.listenForIncoming();
    } catch (error) {
      console.error("Error connecting to MQTT broker:", { error, url });
      throw error;
    }
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

  public getIsConnected = () => this.client !== null;

  /**
   * Define an output to indicate the data that this agent produces. Define an output, and call
   * send(outputName, content, options) to send messages.
   * When sending, the name of the output will be prefixed with the agent type and id, to create a unique routing key.
   * @param {output} output
   */
  createOutput = async (name: string) => {
    const definition: PlugDefinition = {
      name,
      routingKey: `${this.agentType}.${this.agentID}.${name}`,
      flowDirection: "out",
    };

    const output = new Output(this.client, definition);
    this.outputs.push(output);

    return output;
  };

  /**
   * Subscribe to messages from another agent, based on a defined input. Note that it is possible
   * to subscribe to the same binding key multiple times, using different inputs.
   * @param {string} bindingKey The binding key to subscribe to.
   * @param {string} inputName The name of the input to use for this. Note that this input must be defined before subscribing.
   */
  createInput = async (name: string) => {
    // Create a new Input
    const definition: PlugDefinition = {
      name,
      routingKey: `+/+/${name}`,
      flowDirection: "in",
    };
    const input = new Input(this.client, definition);

    input.subscribe();

    this.inputs.push(input);

    return input;
  };
}

const topicHasPlugName = (topic: string, plugName: string) =>
  topic.split(`/`)[2] === plugName;

export default TetherAgent;
