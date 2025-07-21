import { InputPlug, OutputPlug, TetherAgent } from ".";
import { topicMatchesPlug } from "./Plug";

describe("building topic strings", () => {
  test("Default Output Plug, no overrides", async () => {
    const agent = await TetherAgent.create("tester");
    const output = new OutputPlug(agent, "somePlugName");
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual("tester/any/somePlugName");
    agent.disconnect();
  });

  test("Default Input Plug, no overrides", async () => {
    const agent = await TetherAgent.create("tester");
    const input = await InputPlug.create(agent, "somePlugName");
    expect(input.getDefinition().name).toEqual("somePlugName");
    expect(input.getDefinition().topic).toEqual("+/+/somePlugName");
    agent.disconnect();
  });

  test("Agent with custom ID, Output Plug with defaults", async () => {
    const agent = await TetherAgent.create("tester", { id: "specialGroup" });
    const output = new OutputPlug(agent, "somePlugName");
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual(
      "tester/specialGroup/somePlugName"
    );
    agent.disconnect();
  });

  test("Agent with custom ID, Output Plug overriding unrelated config still gets custom ID", async () => {
    const agent = await TetherAgent.create("tester", { id: "specialGroup" });
    const output = new OutputPlug(agent, "somePlugName", {
      publishOptions: { qos: 2 },
    });
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual(
      "tester/specialGroup/somePlugName"
    );
    agent.disconnect();
  });

  test("Agent with custom ID, Output Plug with custom still overrides", async () => {
    const agent = await TetherAgent.create("tester", {
      id: "originalSpecialGroup",
    });
    const output = new OutputPlug(agent, "somePlugName", {
      id: "overrideOnPlugCreation",
    });
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual(
      "tester/overrideOnPlugCreation/somePlugName"
    );
    agent.disconnect();
  });

  test("Agent with custom ID, Input Plug with defaults; still generic", async () => {
    const agent = await TetherAgent.create("tester", { id: "specialGroup" });
    const input = await InputPlug.create(agent, "somePlugName");
    expect(input.getDefinition().name).toEqual("somePlugName");
    expect(input.getDefinition().topic).toEqual("+/+/somePlugName");
    agent.disconnect();
  });

  test("Override ID and/or Role when creating Input", async () => {
    const agent = await TetherAgent.create("tester");

    const inputCustomID = await InputPlug.create(agent, "somePlugName", {
      id: "stillSpecial",
    });
    expect(inputCustomID.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomID.getDefinition().topic).toEqual(
      "+/stillSpecial/somePlugName"
    );

    const inputCustomRole = await InputPlug.create(agent, "somePlugName", {
      role: "specialRole",
    });
    expect(inputCustomRole.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomRole.getDefinition().topic).toEqual(
      "specialRole/+/somePlugName"
    );

    const inputCustomBoth = await InputPlug.create(agent, "somePlugName", {
      id: "id2",
      role: "role2",
    });
    expect(inputCustomBoth.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomBoth.getDefinition().topic).toEqual(
      "role2/id2/somePlugName"
    );

    agent.disconnect();
  });
});

describe("matching topics to plugs", () => {
  test("if Plug specified full topic, i.e. no wildcards, then only exact matches", () => {
    const plugDefinedTopic = "someType/someGroup/somePlugName";

    expect(
      topicMatchesPlug(plugDefinedTopic, "someType/someGroup/somePlugName")
    ).toBeTruthy();

    expect(
      topicMatchesPlug(plugDefinedTopic, "other/otherGroup/somePlugName")
    ).toBeFalsy();
  });

  test("if ONLY Plug Name specified, match any with same PlugName", () => {
    const plugDefinedTopic = "+/+/somePlugName";
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/something/somePlugName")
    ).toBeTruthy();
  });

  test("if ONLY Plug Name specified, match any with same PlugName", () => {
    const plugDefinedTopic = "+/+/somePlugName";
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/something/somePlugName")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/something/somethingElse")
    ).toBeFalsy();
  });

  test("if AgentType and PlugName specified, but not GroupOrId, then match ONLY when these match", () => {
    const plugDefinedTopic = "specificAgent/+/plugName";

    expect(
      topicMatchesPlug(plugDefinedTopic, "specificAgent/anything/plugName")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "specificAgent/somethingElse/plugName")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "differentAgent/anything/plugName")
    ).toBeFalsy();
  });

  test("# wildcard should match any topic", () => {
    const plugDefinedTopic = "#";
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/something/something")
    ).toBeTruthy();
  });

  test("specific use case: agentType specified, no group/ID, plug name", () => {
    const plugDefinedTopic = "LidarConsolidation/+/trackedPoints";

    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "LidarConsolidation/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b/clusters"
      )
    ).toBeFalsy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "LidarConsolidation/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b/trackedPoints"
      )
    ).toBeTruthy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "SomethingElse/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b/trackedPoints"
      )
    ).toBeFalsy();
  });

  test("if GroupOrId and PlugName specified, but not AgentType, then match when these match", () => {
    const plugDefinedTopic = "+/specificGroupOrId/plugName";

    expect(
      topicMatchesPlug(plugDefinedTopic, "someAgent/specificGroupOrId/plugName")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "anotherAgent/specificGroupOrId/plugName"
      )
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "someAgent/wrongGroup/plugName")
    ).toBeFalsy();
  });

  test("if Plug Name was never specified, throw an Error", () => {
    const plugDefinedTopic = "something/something/+";
    try {
      expect(
        topicMatchesPlug(plugDefinedTopic, "anything/anything/anything")
      ).toThrow();
    } catch (e) {
      //
    }
  });
});
