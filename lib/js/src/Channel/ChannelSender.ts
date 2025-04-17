import { IClientPublishOptions } from "async-mqtt";
import { Channel, containsWildcards } from ".";
import {
  encode,
  logger,
  parseAgentIdOrGroup,
  parseAgentRole,
  parseChannelName,
  TetherAgent,
} from "..";

export class ChannelSender<T> extends Channel {
  private publishOptions: IClientPublishOptions;

  constructor(
    agent: TetherAgent,
    channelName: string,
    options?: {
      overrideTopic?: string;
      id?: string;
      publishOptions?: IClientPublishOptions;
    }
  ) {
    super(agent, {
      name: channelName,
      topic:
        options?.overrideTopic ||
        buildOutputTopic(
          channelName,
          agent.getConfig().role,
          options?.id || agent.getConfig().id
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

  /** Do NOT encode the content (assume it's already encoded), and then publish */
  sendRaw = async (content?: Uint8Array) => {
    if (!this.agent.getIsConnected()) {
      throw Error(
        "trying to send without connection; not possible until connected"
      );
    }
    try {
      logger.debug("Sending on topic", this.definition.topic, "with options", {
        ...this.publishOptions,
      });
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
  };

  /** Automatically encode the content as MsgPack, and publish using the ChannelSender topic */
  send = async (content?: T) => {
    if (!this.agent.getIsConnected()) {
      throw Error(
        "trying to send without connection; not possible until connected"
      );
    }
    try {
      const buffer = encode(content);
      this.sendRaw(buffer);
    } catch (e) {
      throw Error(`Error encoding content: ${e}`);
    }
  };
}

const buildOutputTopic = (
  channelName: string,
  specifyRole: string,
  specifyID?: string
): string => {
  const role = specifyRole;
  if (specifyID) {
    return `${role}/${channelName}/${specifyID}`;
  } else {
    return `${role}/${channelName}`;
  }
};
