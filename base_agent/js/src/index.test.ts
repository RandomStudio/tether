import { topicMatchesPlug } from ".";

describe("matching topics to plugs", () => {
  test("if Plug specified full topic, i.e. no wildcards, then only exact matches", () => {
    const plugTopic = "someType/someGroup/somePlugName";

    expect(
      topicMatchesPlug(plugTopic, "someType/someGroup/somePlugName")
    ).toBeTruthy();

    expect(
      topicMatchesPlug(plugTopic, "other/otherGroup/somePlugName")
    ).toBeFalsy();
  });

  test("if ONLY Plug Name specified, match any with same PlugName", () => {
    const plugTopic = "+/+/somePlugName";
    expect(
      topicMatchesPlug(plugTopic, "something/something/somePlugName")
    ).toBeTruthy();
  });
});
