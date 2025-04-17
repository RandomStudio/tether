import { TetherAgent, logger } from "../";
import { ChannelDefinition } from "../types";
import { ChannelReceiver } from "./ChannelReceiver";
import { ChannelSender } from "./ChannelSender";

export class Channel {
  protected definition: ChannelDefinition;
  protected agent: TetherAgent;

  constructor(agent: TetherAgent, definition: ChannelDefinition) {
    this.agent = agent;

    logger.debug("Channel super definition:", JSON.stringify(definition));
    this.definition = definition;
  }

  public getDefinition = () => this.definition;
}

export const containsWildcards = (topicOrPart: string) =>
  topicOrPart.includes("+") || topicOrPart.includes("#");

export const parseChannelName = (topic: string) => topic.split(`/`)[1];
export const parseAgentIdOrGroup = (topic: string) => topic.split(`/`)[2];
export const parseAgentRole = (topic: string) => topic.split(`/`)[0];
