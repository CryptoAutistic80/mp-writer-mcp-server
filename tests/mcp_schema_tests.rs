use mp_writer_mcp_server::features::mcp::schemas::build_tool_schemas;

#[test]
fn search_and_fetch_tools_are_registered() {
    let (definitions, input_schemas) = build_tool_schemas();

    let search_tool = definitions
        .iter()
        .find(|tool| tool.name == "search")
        .expect("search tool definition present");
    let fetch_tool = definitions
        .iter()
        .find(|tool| tool.name == "fetch")
        .expect("fetch tool definition present");

    assert!(input_schemas.contains_key("search"));
    assert!(input_schemas.contains_key("fetch"));

    assert_eq!(search_tool.title, "Search Parliament data");
    assert_eq!(fetch_tool.title, "Fetch Parliament records");
}
