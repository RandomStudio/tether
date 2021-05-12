import { Connection, Channel, connect, Options, ConsumeMessage } from "amqplib";

interface SubscriptionInfo {
  queue: string;
  consumerTag: string;
}

export interface InputDefinition {
  name: string;
  routingKey: string;
}

export interface OutputDefinition {
  name: string;
  routingKey: string;
}

export interface AgentInfo {
  agentType: string;
  agentID: string;
  inputs: InputDefinition[];
  outputs: OutputDefinition[];
}

interface Exchange {
  name: string;
  type: string;
  options?: Options.AssertExchange;
}

const defaultExchange: Exchange = {
  name: "amq.topic",
  type: "topic",
  // options: {
  //   durable: false,
  //   autoDelete: false,
  // },
};

export class Input {
  private subscription: SubscriptionInfo;
  private definition: InputDefinition;

  constructor(definition: InputDefinition) {
    this.definition = definition;
    console.log("created Input:", definition);
  }

  public setSubscription = (subscription: SubscriptionInfo) => {
    this.subscription = subscription;
  };

  public getSubscription = () => this.subscription;

  public getDefinition = () => this.definition;
}

export class Output {
  private channel: Channel;
  private definition: OutputDefinition;
  private agentType: string;
  private agentID: string;

  constructor(
    channel: Channel,
    definition: OutputDefinition,
    agentType: string,
    agentID: string
  ) {
    this.definition = definition;
    console.log("created Output:", definition);
    this.agentType = agentType;
    this.agentID = agentID;
    this.channel = channel;
  }

  public publish = async (content: Buffer, options?: Options.Publish) => {
    this.channel.publish(
      defaultExchange.name,
      this.definition.routingKey,
      Buffer.from(content),
      {
        ...options,
        contentType: "application/msgpack",
        headers: {
          agentID: this.agentID,
          agentType: this.agentType,
        },
      }
    );
  };

  public getDefinition = () => this.definition;
}

export default class TetherAgent {
  private agentType: string = null;
  private agentID: string = null;

  private connection: Connection = null;
  private channel: Channel | null;

  private inputs: Input[] = [];
  private outputs: Output[] = [];

  constructor(agentType: string, agentID: string) {
    this.agentType = agentType;
    this.agentID = agentID;

    this.channel = null;
  }

  private getConnection = async () =>
    this.connection || (await connect("amqp://localhost:5672"));

  private getChannel = async () => {
    if (this.channel) {
      return this.channel;
    } else {
      this.connection = await this.getConnection();

      const channel = await this.connection.createChannel();
      console.log("channel OK");

      // Create default exchange
      await channel.assertExchange(
        defaultExchange.name,
        defaultExchange.type,
        defaultExchange.options
      );
      console.log("exchange OK");

      this.channel = channel; // save so next call will return immediately

      return channel;
    }
  };

  /**
   * Define an output to indicate the data that this agent produces. Define an output, and call
   * send(outputName, content, options) to send messages.
   * When sending, the name of the output will be prefixed with the agent type and id, to create a unique routing key.
   * @param {output} output
   */
  createOutput = async (name: string, routingKey?: string) => {
    const definition: OutputDefinition = {
      name,
      routingKey: routingKey || `${this.agentType}.${this.agentID}.${name}`,
    };

    const channel = await this.getChannel();
    const output = new Output(
      channel,
      definition,
      this.agentType,
      this.agentID
    );
    this.outputs.push(output);

    return output;
  };

  /**
   * Subscribe to messages from another agent, based on a defined input. Note that it is possible
   * to subscribe to the same binding key multiple times, using different inputs.
   * @param {string} bindingKey The binding key to subscribe to.
   * @param {string} inputName The name of the input to use for this. Note that this input must be defined before subscribing.
   */
  createInput = async (
    name: string,
    onMessage: (msg: ConsumeMessage, ...args: any) => void,
    routingKey?: string
  ) => {
    try {
      // Create a new Input
      const definition: InputDefinition = {
        name,
        routingKey: routingKey || `#.${name}`,
      };
      const input = new Input(definition);

      // Create an exclusive queue for the Subscription.
      const channel = await this.getChannel();
      const qResult = await channel.assertQueue("", { exclusive: true });
      const { queue } = qResult;

      // Bind the queue to the exchange.
      await this.channel.bindQueue(
        queue,
        defaultExchange.name,
        definition.routingKey
      );

      // Start consuming, using the callback defined in the input.
      const channelResult = await this.channel.consume(queue, onMessage, {
        noAck: true,
      });

      const { consumerTag } = channelResult;
      const subscription: SubscriptionInfo = {
        queue,
        consumerTag,
      };
      // Associate subscription info (including consumerTag to allow stopping consumption) with Input
      input.setSubscription(subscription);

      // Store the Input
      this.inputs.push(input);
    } catch (err) {
      console.error(err);
    }
  };
}
