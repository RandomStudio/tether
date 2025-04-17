<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import {
    TetherAgent,
    BROWSER,
    ChannelSender,
    encode,
    ChannelReceiver,
    decode,
  } from "tether-agent";

  interface CustomMessage {
    timestamp: number;
    topic: string;
    contents: any;
  }

  let connected = $state(false);
  let sender: ChannelSender<number> | null = $state(null);

  let agent: TetherAgent;

  let messagesReceived: CustomMessage[] = $state([]);

  onMount(async () => {
    agent = await TetherAgent.create("svelteTester", {
      brokerOptions: BROWSER,
    });

    sender = agent.createSender("randomNumbers");

    const receiver = await agent.createReceiver<number>("everything", {
      overrideTopic: "#",
    });
    receiver.on("message", (payload, topic) => {
      const contents = payload;
      messagesReceived = [
        ...messagesReceived,
        {
          timestamp: Date.now(),
          topic,
          contents,
        },
      ];
    });

    connected = agent.getIsConnected();
  });

  onDestroy(async () => {
    await agent.disconnect();
    connected = false;
  });
</script>

<h1>Tether Tester</h1>
<div>
  {#if connected}
    <div>
      <h2>Tether Connected @ localhost</h2>

      <div>
        <h3>Sending</h3>
        <button
          onclick={async () => {
            if (sender) {
              const randomNumber = Math.round(Math.random() * 10000);
              await sender.send(randomNumber);
            } else {
              throw Error("ChannelSender does not exist");
            }
          }}>Send new a message</button
        >
      </div>

      <div>
        <h3>Received {messagesReceived.length} messages</h3>
        <ul>
          {#each messagesReceived as m}
            <li>{m.timestamp} on "{m.topic}": {m.contents}</li>
          {/each}
        </ul>
      </div>
    </div>
  {:else}
    <div>Tether not (yet) connected...</div>
  {/if}
</div>
