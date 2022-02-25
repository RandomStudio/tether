import { AsyncMqttClient } from "async-mqtt";
import { logger } from ".";
import { MessageCallback, PlugDefinition } from "./types";
import { Buffer } from "buffer";

class Plug {
  protected definition: PlugDefinition;
  protected client: AsyncMqttClient | null;

  constructor(client: AsyncMqttClient, definition: PlugDefinition) {
    this.client = client;
    this.definition = definition;
  }

  public getDefinition = () => this.definition;
}

type MessageCallbackListIterm = {
  cb: MessageCallback;
  once: boolean;
};
export class Input extends Plug {
  private onMessageCallbacksList: MessageCallbackListIterm[];

  constructor(client: AsyncMqttClient, definition: PlugDefinition) {
    super(client, definition);
    this.onMessageCallbacksList = [];
  }

  public onMessage(cb: MessageCallback) {
    this.onMessageCallbacksList.push({ cb, once: false });
  }

  public onMessageOnce(cb: MessageCallback) {
    this.onMessageCallbacksList.push({ cb, once: true });
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

  emitMessage = (payload: Buffer, topic: string) => {
    this.onMessageCallbacksList.forEach((i) => {
      i.cb.call(this, payload, topic);
    });
    // And delete any "once only" callbacks
    this.onMessageCallbacksList = this.onMessageCallbacksList.filter(
      (i) => i.once === false
    );
  };
}

export class Output extends Plug {
  publish = async (content?: Buffer | Uint8Array) => {
    if (this.client === null) {
      logger.error(
        "trying to send without connection; not possible until connected"
      );
    } else {
      try {
        if (content === undefined) {
          this.client.publish(this.definition.topic, Buffer.from([]));
        } else if (content instanceof Uint8Array) {
          this.client.publish(this.definition.topic, Buffer.from(content));
        } else {
          this.client.publish(this.definition.topic, content);
        }
      } catch (e) {
        logger.error("Error publishing message:", e);
      }
    }
  };
}
