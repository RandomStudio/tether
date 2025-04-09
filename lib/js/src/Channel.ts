import { IClientPublishOptions, IClientSubscribeOptions } from "async-mqtt";
import { TetherAgent, logger } from ".";
import { ChannelDefinition } from "./types";
import { Buffer } from "buffer";
import EventEmitter from "events";

declare interface Channel {
  on(
    event: "message",
    listener: (payload: Buffer, topic: string) => void,
  ): this;
  on(event: string, listener: Function): this;
}

class Channel extends EventEmitter {
  protected definition: ChannelDefinition;
  protected agent: TetherAgent;

  constructor(agent: TetherAgent, definition: ChannelDefinition) {
    super(); // For EventEmitter
    this.agent = agent;

    logger.debug("Channel super definition:", JSON.stringify(definition));
    this.definition = definition;
  }

  public getDefinition = () => this.definition;
}

export class ChannelReceiver extends Channel {
  public static async create(
    agent: TetherAgent,
    channelName: string,
    options?: {
      overrideTopic?: string;
      id?: string;
      role?: string;
      subscribeOptions?: IClientSubscribeOptions;
    },
  ) {
    const instance = new ChannelReceiver(agent, channelName, options || {});

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
    },
  ) {
    super(agent, {
      name: channelName,
      topic:
        options?.overrideTopic ||
        buildInputTopic(
          channelName,
          options.role,
          options.id ?? agent.getConfig().id,
        ),
    });
    if (agent.getConfig().autoConnect === true && !agent.getIsConnected()) {
      throw Error(
        "trying to create an Input before client is connected; autoConnect? " +
          agent.getConfig().autoConnect,
      );
    }
  }

  private subscribe = async (options?: IClientSubscribeOptions) => {
    if (this.agent.getClient() === null) {
      throw Error("agent client not connected");
    }
    try {
      logger.debug(
        "Attempting subscribtion to topic",
        this.definition.topic,
        `for Channel Input "${this.getDefinition().name}"...`,
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
      `for Channel Input "${this.getDefinition().name}"`,
    );
    this.agent.getClient()?.on("message", (topic, payload) => {
      if (topicMatchesChannel(this.definition.topic, topic)) {
        this.emit("message", payload, topic);
      }
    });
  };
}

export class ChannelSender extends Channel {
  private publishOptions: IClientPublishOptions;

  constructor(
    agent: TetherAgent,
    channelName: string,
    options?: {
      overrideTopic?: string;
      id?: string;
      publishOptions?: IClientPublishOptions;
    },
  ) {
    super(agent, {
      name: channelName,
      topic:
        options?.overrideTopic ||
        buildOutputTopic(
          channelName,
          agent.getConfig().role,
          options?.id || agent.getConfig().id,
        ),
    });
    this.publishOptions = options?.publishOptions || {
      retain: false,
      qos: 1,
    };
    if (channelName === undefined) {
      throw Error("No name provided for Output");
    }
    if (agent.getConfig().autoConnect === true && !agent.getIsConnected()) {
      throw Error("trying to create an Output before client is connected");
    }
  }

  send = async (content?: Buffer | Uint8Array) => {
    if (!this.agent.getIsConnected()) {
      throw Error(
        "trying to send without connection; not possible until connected",
      );
    } else {
      try {
        logger.debug(
          "Sending on topic",
          this.definition.topic,
          "with options",
          { ...this.publishOptions },
        );
        if (content === undefined) {
          this.agent
            .getClient()
            ?.publish(
              this.definition.topic,
              Buffer.from([]),
              this.publishOptions,
            );
        } else if (content instanceof Uint8Array) {
          this.agent
            .getClient()
            ?.publish(
              this.definition.topic,
              Buffer.from(content),
              this.publishOptions,
            );
        } else {
          this.agent
            .getClient()
            ?.publish(this.definition.topic, content, this.publishOptions);
        }
      } catch (e) {
        logger.error("Error publishing message:", e);
      }
    }
  };
}

export const topicMatchesChannel = (
  channelGeneratedTopic: string,
  incomingTopic: string,
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
    parseAgentRole(channelGeneratedTopic),
  )
    ? parseAgentRole(channelGeneratedTopic) === parseAgentRole(incomingTopic)
    : true;

  // if Agent ID or Group specified, see if this matches, otherwise pass all AgentIdOrGroup as matches
  // e.g. +/channelName/specifiedID
  const agentIdOrGroupMatches = !containsWildcards(
    parseAgentIdOrGroup(channelGeneratedTopic),
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

const containsWildcards = (topicOrPart: string) =>
  topicOrPart.includes("+") || topicOrPart.includes("#");

const buildInputTopic = (
  channelName: string,
  specifyRole?: string,
  specifyID?: string,
): string => {
  const role = specifyRole || "+";
  if (specifyID) {
    return `${role}/${channelName}/${specifyID}`;
  } else {
    return `${role}/${channelName}/#`;
  }
};

const buildOutputTopic = (
  channelName: string,
  specifyRole: string,
  specifyID?: string,
): string => {
  const role = specifyRole;
  if (specifyID) {
    return `${role}/${channelName}/${specifyID}`;
  } else {
    return `${role}/${channelName}`;
  }
};

export const parseChannelName = (topic: string) => topic.split(`/`)[1];
export const parseAgentIdOrGroup = (topic: string) => topic.split(`/`)[2];
export const parseAgentRole = (topic: string) => topic.split(`/`)[0];
