import { IClientSubscribeOptions } from "async-mqtt";
import { Channel, containsWildcards } from ".";
import {
  decode,
  logger,
  parseAgentIdOrGroup,
  parseAgentRole,
  parseChannelName,
  TetherAgent,
} from "..";

export interface ReceiverOptions {
  overrideTopic?: string;
  id?: string;
  role?: string;
  subscribeOptions?: IClientSubscribeOptions;
}

type ReceiverCallback<T> = (payload: T, topic: string) => void;
export class ChannelReceiver<T> extends Channel {
  private callbacks: ReceiverCallback<T>[];

  public static async create<T>(
    agent: TetherAgent,
    channelName: string,
    options?: ReceiverOptions
  ) {
    const instance = new ChannelReceiver<T>(agent, channelName, options || {});

    if (agent.getConfig().autoConnect === false) {
      logger.warn(
        "Agent had autoConnect set to FALSE; will not attempt subscription - fine for testing, bad for anything else!"
      );
      return instance;
    }

    try {
      await instance.subscribe(options?.subscribeOptions || { qos: 1 });
      logger.info("subscribed OK to", instance.definition.topic);
    } catch (e) {
      logger.error("failed to subscribe:", e);
    }

    return instance;
  }

  private constructor(
    agent: TetherAgent,
    channelName: string,
    options: {
      overrideTopic?: string;
      id?: string;
      role?: string;
      subscribeOptions?: IClientSubscribeOptions;
    }
  ) {
    super(agent, {
      name: channelName,
      topic:
        options?.overrideTopic ||
        buildInputTopic(
          channelName,
          options.role,
          options.id ?? agent.getConfig().id
        ),
    });
    this.callbacks = [];
    if (agent.getConfig().autoConnect === true && !agent.getIsConnected()) {
      throw Error(
        "trying to create an Input before client is connected; autoConnect? " +
          agent.getConfig().autoConnect
      );
    }
  }

  public on(event: string, cb: ReceiverCallback<T>) {
    if (event !== "message") {
      throw Error(`only "message" events can be registered`);
    }
    this.callbacks.push(cb);
  }

  private subscribe = async (options?: IClientSubscribeOptions) => {
    if (this.agent.getClient() === null) {
      throw Error("agent client not connected");
    }
    try {
      logger.debug(
        "Attempting subscribtion to topic",
        this.definition.topic,
        `for Channel Input "${this.getDefinition().name}"...`
      );
      if (options === undefined) {
        await this.agent.getClient()?.subscribe(this.definition.topic);
      } else {
        await this.agent.getClient()?.subscribe(this.definition.topic, options);
      }
    } catch (e) {
      logger.error("Error subscribing ", e);
      throw Error("Subscribe error: " + e);
    }
    logger.debug(
      "subscribed to topic",
      this.definition.topic,
      `for Channel Input "${this.getDefinition().name}"`
    );
    this.agent.getClient()?.on("message", (topic, payload) => {
      if (topicMatchesChannel(this.definition.topic, topic)) {
        // this.emit("message", payload, topic);
        logger.debug("Received", payload);
        this.callbacks.forEach((cb) => {
          const decoded =
            payload.length === 0 ? payload : (decode(payload) as T);
          cb.call(this, decoded as T, topic);
        });
      }
    });
  };
}

const buildInputTopic = (
  channelName: string,
  specifyRole?: string,
  specifyID?: string
): string => {
  const role = specifyRole || "+";
  if (specifyID) {
    return `${role}/${channelName}/${specifyID}`;
  } else {
    return `${role}/${channelName}/#`;
  }
};

export const topicMatchesChannel = (
  channelGeneratedTopic: string,
  incomingTopic: string
): boolean => {
  if (channelGeneratedTopic == "#") {
    return true;
  }

  if (!containsWildcards(channelGeneratedTopic)) {
    // No wildcards at all in full topic e.g. specified/channelName/alsoSpecified ...
    return channelGeneratedTopic === incomingTopic;
    // ... Then MATCH only if the defined topic and incoming topic match EXACTLY
  }

  const incomingChannelName = parseChannelName(incomingTopic);
  const topicDefinedChannelName = parseChannelName(channelGeneratedTopic);

  // if (!containsWildcards(incomingChannelName)) {
  if (
    containsWildcards(parseAgentRole(channelGeneratedTopic)) &&
    containsWildcards(parseAgentIdOrGroup(channelGeneratedTopic))
    // if ONLY the Channel Name was specified (which is the default), then MATCH
    // anything that matches the Chanel Name, regardless of the rest
  ) {
    return topicDefinedChannelName === incomingChannelName;
  }

  // If either the Role or ID/Group was specified, check these as well...

  // if Role specified, see if this matches, otherwise pass all AgentTypes as matches
  // e.g. specified/channelName
  const agentTypeMatches = !containsWildcards(
    parseAgentRole(channelGeneratedTopic)
  )
    ? parseAgentRole(channelGeneratedTopic) === parseAgentRole(incomingTopic)
    : true;

  // if Agent ID or Group specified, see if this matches, otherwise pass all AgentIdOrGroup as matches
  // e.g. +/channelName/specifiedID
  const agentIdOrGroupMatches = !containsWildcards(
    parseAgentIdOrGroup(channelGeneratedTopic)
  )
    ? parseAgentIdOrGroup(channelGeneratedTopic) ===
      parseAgentIdOrGroup(incomingTopic)
    : true;

  return (
    agentTypeMatches &&
    agentIdOrGroupMatches &&
    incomingChannelName === topicDefinedChannelName
  );
  // } else {
  //     // something/+ is not allowed for Channels - is that true??
  //     throw Error(
  //       "No ChannelName was specified for this Channel: " + channelGeneratedTopic,
  //     );
  //   }
};
