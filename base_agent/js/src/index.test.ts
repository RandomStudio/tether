import { InputPlug, OutputPlug, TetherAgent } from ".";
import { topicMatchesPlug } from "./Plug";
import { describe, test, expect } from "@jest/globals";

describe("building topic strings", () => {
  test("Default Output Plug, no ID", async () => {
    const agent = await TetherAgent.create("tester");
    const output = new OutputPlug(agent, "somePlugName");
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual("tester/somePlugName");
    agent.disconnect();
  });

  test("Default Input Plug, no ID", async () => {
    const agent = await TetherAgent.create("tester");
    const input = await InputPlug.create(agent, "somePlugName");
    expect(input.getDefinition().name).toEqual("somePlugName");
    expect(input.getDefinition().topic).toEqual("+/somePlugName/#");
    agent.disconnect();
  });

  test("Agent with custom ID, Output Plug with defaults", async () => {
    const agent = await TetherAgent.create("tester", { id: "specialGroup" });
    const output = new OutputPlug(agent, "somePlugName");
    expect(output.getDefinition().name).toEqual("somePlugName");
    expect(output.getDefinition().topic).toEqual(
      "tester/somePlugName/specialGroup"
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
      "tester/somePlugName/overrideOnPlugCreation"
    );
    agent.disconnect();
  });

  test("Agent with custom ID, Input Plug with defaults; still generic", async () => {
    const agent = await TetherAgent.create("tester", { id: "specialGroup" });
    const input = await InputPlug.create(agent, "somePlugName");
    expect(input.getDefinition().name).toEqual("somePlugName");
    expect(input.getDefinition().topic).toEqual("+/somePlugName/#");
    agent.disconnect();
  });

  test("Override ID and/or Role when creating Input", async () => {
    const agent = await TetherAgent.create("tester");

    const inputCustomID = await InputPlug.create(agent, "somePlugName", {
      id: "specialID",
    });
    expect(inputCustomID.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomID.getDefinition().topic).toEqual(
      "+/somePlugName/specialID"
    );

    const inputCustomRole = await InputPlug.create(agent, "somePlugName", {
      role: "specialRole",
    });
    expect(inputCustomRole.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomRole.getDefinition().topic).toEqual(
      "specialRole/somePlugName/#"
    );

    const inputCustomBoth = await InputPlug.create(agent, "somePlugName", {
      id: "id2",
      role: "role2",
    });
    expect(inputCustomBoth.getDefinition().name).toEqual("somePlugName");
    expect(inputCustomBoth.getDefinition().topic).toEqual(
      "role2/somePlugName/id2"
    );

    agent.disconnect();
  });
});

describe("matching topics to plugs", () => {
  test("if Plug specified full topic, i.e. no wildcards, then only exact matches", () => {
    const plugDefinedTopic = "someType/somePlugName/someID";

    expect(
      topicMatchesPlug(plugDefinedTopic, "someType/somePlugName/someID")
    ).toBeTruthy();

    expect(
      topicMatchesPlug(plugDefinedTopic, "other/somePlugName/otherGroup")
    ).toBeFalsy();
  });

  test("if ONLY Plug Name specified, match any with same PlugName", () => {
    const plugDefinedTopic = "+/somePlugName/#";
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/somePlugName")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/somePlugName/something")
    ).toBeTruthy();
  });

  test("if ONLY Plug Name specified, match any with same PlugName", () => {
    const plugDefinedTopic = "+/somePlugName/#";
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/somePlugName/something")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "something/someOtherPlugName.something"
      )
    ).toBeFalsy();
  });

  test("if AgentRole and PlugName specified, but not GroupOrId, then match ONLY when these match", () => {
    const plugDefinedTopic = "specificAgent/plugName/#";

    expect(
      topicMatchesPlug(plugDefinedTopic, "specificAgent/plugName")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "specificAgent/plugName/anything")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "specificAgent/plugName/somethingElse")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "differentAgent/plugName/anything")
    ).toBeFalsy();
  });

  test("# wildcard should match any topic", () => {
    const plugDefinedTopic = "#";
    expect(
      topicMatchesPlug(plugDefinedTopic, "something/something/something")
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "not/event/tether/standard")
    ).toBeTruthy();
  });

  test("specific use case: agentRole specified, no group/ID, plug name", () => {
    const plugDefinedTopic = "LidarConsolidation/trackedPoints/#";

    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "LidarConsolidation/clusters/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b"
      )
    ).toBeFalsy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "LidarConsolidation/trackedPoints/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b"
      )
    ).toBeTruthy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "SomethingElse/trackedPoints/e933b82f-cb0d-4f91-a4a7-5625ce3ed20b"
      )
    ).toBeFalsy();
  });

  test("if GroupOrId and PlugName specified, but not AgentRole, then match when these match", () => {
    const plugDefinedTopic = "+/plugName/specificGroupOrId";

    expect(
      topicMatchesPlug(plugDefinedTopic, "someAgentRole/plugName")
    ).toBeFalsy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "someAgentRole/plugName/specificGroupOrId"
      )
    ).toBeTruthy();
    expect(
      topicMatchesPlug(
        plugDefinedTopic,
        "anotherAgent/plugName/specificGroupOrId"
      )
    ).toBeTruthy();
    expect(
      topicMatchesPlug(plugDefinedTopic, "someAgent/plugName/wrongGroup")
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
