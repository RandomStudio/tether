import { ChannelReceiver, ChannelSender, TetherAgent } from ".";
import { topicMatchesChannel } from "./Channel";
import { describe, test, expect } from "@jest/globals";

describe("building topic strings", () => {
  test("Default Channel Output, no ID", async () => {
    const agent = await TetherAgent.create("tester", { autoConnect: false });
    const output = new ChannelSender(agent, "someChannelName");
    expect(output.getDefinition().name).toEqual("someChannelName");
    expect(output.getDefinition().topic).toEqual("tester/someChannelName");
  });

  test("Default Channel Input, no ID", async () => {
    const agent = await TetherAgent.create("tester", { autoConnect: false });
    const input = await ChannelReceiver.create(agent, "someChannelName");
    expect(input.getDefinition().name).toEqual("someChannelName");
    expect(input.getDefinition().topic).toEqual("+/someChannelName/#");
  });

  test("Agent with custom ID, Channel Output with defaults", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
      id: "specialGroup",
    });
    const output = new ChannelSender(agent, "someChannelName");
    expect(output.getDefinition().name).toEqual("someChannelName");
    expect(output.getDefinition().topic).toEqual(
      "tester/someChannelName/specialGroup",
    );
  });

  test("Agent with custom ID, Channel Output with custom still overrides", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
      id: "originalSpecialGroup",
    });
    const output = new ChannelSender(agent, "someChannelName", {
      id: "overrideOnChannelCreation",
    });
    expect(output.getDefinition().name).toEqual("someChannelName");
    expect(output.getDefinition().topic).toEqual(
      "tester/someChannelName/overrideOnChannelCreation",
    );
  });

  test("Agent with custom ID, Channel Input with defaults; NOT generic", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
      id: "specialGroup",
    });
    const input = await ChannelReceiver.create(agent, "someChannelName");
    expect(input.getDefinition().name).toEqual("someChannelName");
    expect(input.getDefinition().topic).toEqual(
      "+/someChannelName/specialGroup",
    );
  });

  test("Override ID and/or Role when creating Input", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
    });

    const inputCustomID = await ChannelReceiver.create(
      agent,
      "someChannelName",
      {
        id: "specialID",
      },
    );
    expect(inputCustomID.getDefinition().name).toEqual("someChannelName");
    expect(inputCustomID.getDefinition().topic).toEqual(
      "+/someChannelName/specialID",
    );

    const inputCustomRole = await ChannelReceiver.create(
      agent,
      "someChannelName",
      {
        role: "specialRole",
      },
    );
    expect(inputCustomRole.getDefinition().name).toEqual("someChannelName");
    expect(inputCustomRole.getDefinition().topic).toEqual(
      "specialRole/someChannelName/#",
    );

    const inputCustomBoth = await ChannelReceiver.create(
      agent,
      "someChannelName",
      {
        id: "id2",
        role: "role2",
      },
    );
    expect(inputCustomBoth.getDefinition().name).toEqual("someChannelName");
    expect(inputCustomBoth.getDefinition().topic).toEqual(
      "role2/someChannelName/id2",
    );
  });
});

describe("matching topics to Channels", () => {
  test("if Channel specified full topic, i.e. no wildcards, then only exact matches", () => {
    const channelDefinedTopic = "someType/someChannelName/someID";

    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "someType/someChannelName/someID",
      ),
    ).toBeTruthy();

    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "other/someChannelName/otherGroup",
      ),
    ).toBeFalsy();
  });

  test("if ONLY Channel Name specified, match any with same ChanelName", () => {
    const channelDefinedTopic = "+/someChannelName/#";
    expect(
      topicMatchesChannel(channelDefinedTopic, "something/someChannelName"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "something/someChannelName/something",
      ),
    ).toBeTruthy();
  });

  test("if ONLY Channel Name specified, match any with same ChannelName", () => {
    const channelDefinedTopic = "+/someChannelName/#";
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "something/someChannelName/something",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "something/someOtherChannelName.something",
      ),
    ).toBeFalsy();
  });

  test("if AgentRole and ChannelName specified, but not GroupOrId, then match ONLY when these match", () => {
    const channelDefinedTopic = "specificAgent/channelName/#";

    expect(
      topicMatchesChannel(channelDefinedTopic, "specificAgent/channelName"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "specificAgent/channelName/anything",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "specificAgent/channelName/somethingElse",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "differentAgent/channelName/anything",
      ),
    ).toBeFalsy();
  });

  test("# wildcard should match any topic", () => {
    const channelDefinedTopic = "#";
    expect(
      topicMatchesChannel(channelDefinedTopic, "something/something/something"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(channelDefinedTopic, "not/event/tether/standard"),
    ).toBeTruthy();
  });

  test("specific use case: agentRole specified, no group/ID, channel name", () => {
    const channelDefinedTopic = "LidarConsolidation/trackedPoints/#";

    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "LidarConsolidation/clusters/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b",
      ),
    ).toBeFalsy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "LidarConsolidation/trackedPoints/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "SomethingElse/trackedPoints/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b",
      ),
    ).toBeFalsy();
  });

  test("if GroupOrId and ChannelName specified, but not AgentRole, then match when these match", () => {
    const channelDefinedTopic = "+/channelName/specificGroupOrId";

    expect(
      topicMatchesChannel(channelDefinedTopic, "someAgentRole/channelName"),
    ).toBeFalsy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "someAgentRole/channelName/specificGroupOrId",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "anotherAgent/channelName/specificGroupOrId",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        channelDefinedTopic,
        "someAgent/channelName/wrongGroup",
      ),
    ).toBeFalsy();
  });

  test("if Channel Name was never specified, throw an Error", () => {
    const channelDefinedTopic = "something/something/+";
    try {
      expect(
        topicMatchesChannel(channelDefinedTopic, "anything/anything/anything"),
      ).toThrow();
    } catch (e) {
      //
    }
  });
});
