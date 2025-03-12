import { IClientPublishOptions, IClientSubscribeOptions } from "async-mqtt";
import { TetherAgent, logger } from ".";
import { PlugDefinition } from "./types";
import { Buffer } from "buffer";
import EventEmitter from "events";

declare interface Plug {
  on(
    event: "message",
    listener: (payload: Buffer, topic: string) => void
  ): this;
  on(event: string, listener: Function): this;
}

class Plug extends EventEmitter {
  protected definition: PlugDefinition;
  protected agent: TetherAgent;

  constructor(agent: TetherAgent, definition: PlugDefinition) {
    super(); // For EventEmitter
    this.agent = agent;

    logger.debug("Plug super definition:", JSON.stringify(definition));
    this.definition = definition;
  }

  public getDefinition = () => this.definition;
}

export class InputPlug extends Plug {
  public static async create(
    agent: TetherAgent,
    name: string,
    options?: {
      overrideTopic?: string;
      id?: string;
      role?: string;
      subscribeOptions?: IClientSubscribeOptions;
    }
  ) {
    const inputPlug = new InputPlug(agent, name, options || {});

    try {
      await inputPlug.subscribe(options?.subscribeOptions || { qos: 1 });
      logger.info("subscribed OK to", inputPlug.definition.topic);
    } catch (e) {
      logger.error("failed to subscribe:", e);
    }

    return inputPlug;
  }

  private constructor(
    agent: TetherAgent,
    name: string,
    options: {
      overrideTopic?: string;
      id?: string;
      role?: string;
      subscribeOptions?: IClientSubscribeOptions;
    }
  ) {
    super(agent, {
      name,
      topic:
        options?.overrideTopic ||
        buildInputPlugTopic(name, options.id, options.role),
    });
    if (!agent.getIsConnected()) {
      throw Error("trying to create an Input before client is connected");
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
        `for Input Plug "${this.getDefinition().name}"...`
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
      `for Input Plug "${this.getDefinition().name}"`
    );
    this.agent.getClient()?.on("message", (topic, payload) => {
      if (topicMatchesPlug(this.definition.topic, topic)) {
        this.emit("message", payload, topic);
      }
    });
  };
}

export class OutputPlug extends Plug {
  private publishOptions: IClientPublishOptions;

  constructor(
    agent: TetherAgent,
    name: string,
    options?: {
      overrideTopic?: string;
      id?: string;
      publishOptions?: IClientPublishOptions;
    }
  ) {
    super(agent, {
      name,
      topic:
        options?.overrideTopic ||
        buildOutputPlugTopic(
          name,
          agent.getConfig().role,
          options ? options.id : agent.getConfig().id
        ),
    });
    this.publishOptions = options?.publishOptions || {
      retain: false,
      qos: 1,
    };
    if (name === undefined) {
      throw Error("No name provided for output");
    }
    if (!agent.getIsConnected()) {
      throw Error("trying to create an Output before client is connected");
    }
  }

  publish = async (content?: Buffer | Uint8Array) => {
    if (!this.agent.getIsConnected()) {
      logger.error(
        "trying to send without connection; not possible until connected"
      );
    } else {
      try {
        logger.debug(
          "Sending on topic",
          this.definition.topic,
          "with options",
          { ...this.publishOptions }
        );
        if (content === undefined) {
          this.agent
            .getClient()
            ?.publish(
              this.definition.topic,
              Buffer.from([]),
              this.publishOptions
            );
        } else if (content instanceof Uint8Array) {
          this.agent
            .getClient()
            ?.publish(
              this.definition.topic,
              Buffer.from(content),
              this.publishOptions
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

export const topicMatchesPlug = (
  plugTopic: string,
  incomingTopic: string
): boolean => {
  if (plugTopic == "#") {
    return true;
  }

  if (!containsWildcards(plugTopic)) {
    // No wildcards at all in full topic e.g. specified/alsoSpecified/plugName ...
    return plugTopic === incomingTopic;
    // ... Then MATCH only if the defined topic and incoming topic match EXACTLY
  }

  const incomingPlugName = parsePlugName(incomingTopic);
  const topicDefinedPlugName = parsePlugName(plugTopic);

  if (!containsWildcards(incomingPlugName)) {
    if (
      containsWildcards(parseAgentType(plugTopic)) &&
      containsWildcards(parseAgentIdOrGroup(plugTopic))
      // if ONLY the Plug Name was specified (which is the default), then MATCH
      // anything that matches the Plug Name, regardless of the rest
    ) {
      return topicDefinedPlugName === incomingPlugName;
    }

    // If either the AgentType or ID/Group was specified, check these as well...

    // if AgentType specified, see if this matches, otherwise pass all AgentTypes as matches
    // e.g. specified/+/plugName
    const agentTypeMatches = !containsWildcards(parseAgentType(plugTopic))
      ? parseAgentType(plugTopic) === parseAgentType(incomingTopic)
      : true;

    // if Agent ID or Group specified, see if this matches, otherwise pass all AgentIdOrGroup as matches
    // e.g. +/specified/plugName
    const agentIdOrGroupMatches = !containsWildcards(
      parseAgentIdOrGroup(plugTopic)
    )
      ? parseAgentIdOrGroup(plugTopic) === parseAgentIdOrGroup(incomingTopic)
      : true;

    return (
      agentTypeMatches &&
      agentIdOrGroupMatches &&
      incomingPlugName === topicDefinedPlugName
    );
  } else {
    // something/something/+ is not allowed for Plugs
    throw Error("No PlugName was specified for this Plug: " + plugTopic);
  }
};

const containsWildcards = (topicOrPart: string) => topicOrPart.includes("+");

const buildInputPlugTopic = (
  plugName: string,
  specifyID?: string,
  specifyRole?: string
): string => {
  const id = specifyID || "+";
  const role = specifyRole || "+";
  return `${role}/${id}/${plugName}`;
};

const buildOutputPlugTopic = (
  plugName: string,
  specifyRole: string,
  specifyID?: string
): string => {
  const id = specifyID || "any";
  const role = specifyRole;
  return `${role}/${id}/${plugName}`;
};

export const parsePlugName = (topic: string) => topic.split(`/`)[2];
export const parseAgentIdOrGroup = (topic: string) => topic.split(`/`)[1];
export const parseAgentType = (topic: string) => topic.split(`/`)[0];
