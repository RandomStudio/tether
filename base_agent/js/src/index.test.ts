import { ChannelInput, ChannelOutput, TetherAgent } from ".";
import { topicMatchesChannel } from "./Channel";
import { describe, test, expect } from "@jest/globals";

describe("building topic strings", () => {
  test("Default Output Plug, no ID", async () => {
    const agent = await TetherAgent.create("tester", { autoConnect: false });
    const output = new ChannelOutput(agent, "somePlugName");
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual("tester/somePlugName");
  });

  test("Default Input Plug, no ID", async () => {
    const agent = await TetherAgent.create("tester", { autoConnect: false });
    const input = await ChannelInput.create(agent, "somePlugName");
    expect(input.getDefinition().name).toEqual("somePlugName");
    expect(input.getDefinition().topic).toEqual("+/somePlugName/#");
  });

  test("Agent with custom ID, Output Plug with defaults", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
      id: "specialGroup",
    });
    const output = new ChannelOutput(agent, "somePlugName");
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual(
      "tester/somePlugName/specialGroup",
    );
  });

  test("Agent with custom ID, Output Plug with custom still overrides", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
      id: "originalSpecialGroup",
    });
    const output = new ChannelOutput(agent, "somePlugName", {
      id: "overrideOnPlugCreation",
    });
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual(
      "tester/somePlugName/overrideOnPlugCreation",
    );
  });

  test("Agent with custom ID, Input Plug with defaults; still generic", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
      id: "specialGroup",
    });
    const input = await ChannelInput.create(agent, "somePlugName");
    expect(input.getDefinition().name).toEqual("somePlugName");
    expect(input.getDefinition().topic).toEqual("+/somePlugName/#");
  });

  test("Override ID and/or Role when creating Input", async () => {
    const agent = await TetherAgent.create("tester", {
      autoConnect: false,
    });

    const inputCustomID = await ChannelInput.create(agent, "somePlugName", {
      id: "specialID",
    });
    expect(inputCustomID.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomID.getDefinition().topic).toEqual(
      "+/somePlugName/specialID",
    );

    const inputCustomRole = await ChannelInput.create(agent, "somePlugName", {
      role: "specialRole",
    });
    expect(inputCustomRole.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomRole.getDefinition().topic).toEqual(
      "specialRole/somePlugName/#",
    );

    const inputCustomBoth = await ChannelInput.create(agent, "somePlugName", {
      id: "id2",
      role: "role2",
    });
    expect(inputCustomBoth.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomBoth.getDefinition().topic).toEqual(
      "role2/somePlugName/id2",
    );
  });
});

describe("matching topics to plugs", () => {
  test("if Plug specified full topic, i.e. no wildcards, then only exact matches", () => {
    const plugDefinedTopic = "someType/somePlugName/someID";

    expect(
      topicMatchesChannel(plugDefinedTopic, "someType/somePlugName/someID"),
    ).toBeTruthy();

    expect(
      topicMatchesChannel(plugDefinedTopic, "other/somePlugName/otherGroup"),
    ).toBeFalsy();
  });

  test("if ONLY Plug Name specified, match any with same PlugName", () => {
    const plugDefinedTopic = "+/somePlugName/#";
    expect(
      topicMatchesChannel(plugDefinedTopic, "something/somePlugName"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(plugDefinedTopic, "something/somePlugName/something"),
    ).toBeTruthy();
  });

  test("if ONLY Plug Name specified, match any with same PlugName", () => {
    const plugDefinedTopic = "+/somePlugName/#";
    expect(
      topicMatchesChannel(plugDefinedTopic, "something/somePlugName/something"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        plugDefinedTopic,
        "something/someOtherPlugName.something",
      ),
    ).toBeFalsy();
  });

  test("if AgentRole and PlugName specified, but not GroupOrId, then match ONLY when these match", () => {
    const plugDefinedTopic = "specificAgent/plugName/#";

    expect(
      topicMatchesChannel(plugDefinedTopic, "specificAgent/plugName"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(plugDefinedTopic, "specificAgent/plugName/anything"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        plugDefinedTopic,
        "specificAgent/plugName/somethingElse",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(plugDefinedTopic, "differentAgent/plugName/anything"),
    ).toBeFalsy();
  });

  test("# wildcard should match any topic", () => {
    const plugDefinedTopic = "#";
    expect(
      topicMatchesChannel(plugDefinedTopic, "something/something/something"),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(plugDefinedTopic, "not/event/tether/standard"),
    ).toBeTruthy();
  });

  test("specific use case: agentRole specified, no group/ID, plug name", () => {
    const plugDefinedTopic = "LidarConsolidation/trackedPoints/#";

    expect(
      topicMatchesChannel(
        plugDefinedTopic,
        "LidarConsolidation/clusters/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b",
      ),
    ).toBeFalsy();
    expect(
      topicMatchesChannel(
        plugDefinedTopic,
        "LidarConsolidation/trackedPoints/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        plugDefinedTopic,
        "SomethingElse/trackedPoints/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b",
      ),
    ).toBeFalsy();
  });

  test("if GroupOrId and PlugName specified, but not AgentRole, then match when these match", () => {
    const plugDefinedTopic = "+/plugName/specificGroupOrId";

    expect(
      topicMatchesChannel(plugDefinedTopic, "someAgentRole/plugName"),
    ).toBeFalsy();
    expect(
      topicMatchesChannel(
        plugDefinedTopic,
        "someAgentRole/plugName/specificGroupOrId",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(
        plugDefinedTopic,
        "anotherAgent/plugName/specificGroupOrId",
      ),
    ).toBeTruthy();
    expect(
      topicMatchesChannel(plugDefinedTopic, "someAgent/plugName/wrongGroup"),
    ).toBeFalsy();
  });

  test("if Plug Name was never specified, throw an Error", () => {
    const plugDefinedTopic = "something/something/+";
    try {
      expect(
        topicMatchesChannel(plugDefinedTopic, "anything/anything/anything"),
      ).toThrow();
    } catch (e) {
      //
    }
  });
});
