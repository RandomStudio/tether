import { AsyncMqttClient } from "async-mqtt";
import EventEmitter from "events";
import { logger } from ".";
import { PlugDefinition } from "./types";

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

export declare interface Input {
  on(
    event: "message",
    listener: (payload: Buffer, topic: string) => void
  ): this;
}

export class Input extends Plug {
  subscribe = async () => {
    if (this.client === null) {
      logger.warn(
        "subscribing to topic before client is connected; this is allowed but you won't receive any messages until connected"
      );
    }
    await this.client.subscribe(this.definition.topic);
    logger.debug("subscribed to topic", this.definition.topic);
  };
}

export class Output extends Plug {
  publish = async (content: Buffer | Uint8Array) => {
    if (this.client === null) {
      logger.error(
        "trying to send without connection; not possible until connected"
      );
    } else {
      if (content instanceof Uint8Array) {
        this.client.publish(this.definition.topic, Buffer.from(content));
      } else {
        this.client.publish(this.definition.topic, content);
      }
    }
  };
}
