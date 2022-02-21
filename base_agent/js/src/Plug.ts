import { AsyncMqttClient } from "async-mqtt";
import { logger } from ".";
import { PlugDefinition } from "./types";

class Plug {
  protected definition: PlugDefinition;
  protected client: AsyncMqttClient | null;

  constructor(client: AsyncMqttClient, definition: PlugDefinition) {
    this.client = client;
    this.definition = definition;
  }

  public getDefinition = () => this.definition;
}

export class Input extends Plug {
  private onMessageCallbacks: Function[];

  constructor(client: AsyncMqttClient, definition: PlugDefinition) {
    super(client, definition);
    this.onMessageCallbacks = [];
  }

  public on(
    _event: "message",
    cb: (payload: Buffer, topic: string) => void
  ): this {
    this.onMessageCallbacks.push(cb);
    return this;
  }
  subscribe = async () => {
    if (this.client === null) {
      logger.warn(
        "subscribing to topic before client is connected; this is allowed but you won't receive any messages until connected"
      );
    }
    await this.client.subscribe(this.definition.topic);
    logger.debug(
      "subscribed to topic",
      this.definition.topic,
      `for Input Plug "${this.getDefinition().name}"`
    );
  };

  emit = (event: "message", payload: Buffer, topic: string) => {
    this.onMessageCallbacks.forEach((cb) => {
      cb.call(this, payload, topic);
    });
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
        try {
          this.client.publish(this.definition.topic, Buffer.from(content));
        } catch (e) {
          logger.error("Error publishing message:", e);
        }
      } else {
        this.client.publish(this.definition.topic, content);
      }
    }
  };
}
