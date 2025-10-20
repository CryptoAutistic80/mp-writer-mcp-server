use std::collections::HashMap;

use serde_json::{Value, json};

use crate::features::mcp::dto::ToolDefinition;

pub fn build_tool_schemas() -> (Vec<ToolDefinition>, HashMap<String, Value>) {
    let mut definitions = Vec::new();
    let mut input_schemas = HashMap::new();

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "search",
        "Search Parliament data",
        "Perform searches across UK Parliament datasets including legislation, bills, and indexed core datasets.",
        json!({
            "type": "object",
            "required": ["target"],
            "properties": {
                "target": {"type": "string", "enum": ["uk_law", "bills", "dataset"]},
                "query": {"type": "string", "minLength": 1},
                "dataset": {"type": "string", "minLength": 1},
                "legislationType": {"type": "string", "enum": ["primary", "secondary", "all"]},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50},
                "enableCache": {"type": "boolean"},
                "applyRelevance": {"type": "boolean"},
                "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0},
                "fuzzyMatch": {"type": "boolean"},
                "house": {"type": "string", "enum": ["commons", "lords"]},
                "session": {"type": "string"},
                "parliamentNumber": {"type": "integer", "minimum": 1},
                "page": {"type": "integer", "minimum": 0},
                "perPage": {"type": "integer", "minimum": 1, "maximum": 100}
            },
            "allOf": [
                {"if": {"properties": {"target": {"const": "uk_law"}}}, "then": {"required": ["query"]}},
                {"if": {"properties": {"target": {"const": "bills"}}}, "then": {"required": ["query"]}},
                {"if": {"properties": {"target": {"const": "dataset"}}}, "then": {"required": ["dataset", "query"]}}
            ],
            "additionalProperties": false
        }),
        Some(json!({
            "description": "Array or object payloads returned by Parliament search endpoints.",
            "oneOf": [
                {"type": "array"},
                {"type": "object"},
                {"type": "string"},
                {"type": "null"}
            ]
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "fetch",
        "Fetch Parliament records",
        "Retrieve detailed Parliament records such as datasets, MP activity, voting records, and constituency lookups.",
        json!({
            "type": "object",
            "required": ["target"],
            "properties": {
                "target": {
                    "type": "string",
                    "enum": [
                        "core_dataset",
                        "bills",
                        "legislation",
                        "mp_activity",
                        "mp_voting_record",
                        "constituency"
                    ]
                },
                "dataset": {"type": "string", "minLength": 1},
                "searchTerm": {"type": "string"},
                "page": {"type": "integer", "minimum": 0},
                "perPage": {"type": "integer", "minimum": 1, "maximum": 100},
                "enableCache": {"type": "boolean"},
                "applyRelevance": {"type": "boolean"},
                "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0},
                "fuzzyMatch": {"type": "boolean"},
                "house": {"type": "string", "enum": ["commons", "lords"]},
                "session": {"type": "string"},
                "parliamentNumber": {"type": "integer", "minimum": 1},
                "mpId": {"type": "integer", "minimum": 1},
                "fromDate": {"type": "string", "format": "date"},
                "toDate": {"type": "string", "format": "date"},
                "billId": {"type": "string"},
                "legislationType": {"type": "string"},
                "title": {"type": "string"},
                "year": {"type": "integer", "minimum": 1800},
                "postcode": {"type": "string", "minLength": 2},
                "limit": {"type": "integer", "minimum": 1, "maximum": 100}
            },
            "allOf": [
                {"if": {"properties": {"target": {"const": "core_dataset"}}}, "then": {"required": ["dataset"]}},
                {"if": {"properties": {"target": {"const": "mp_activity"}}}, "then": {"required": ["mpId"]}},
                {"if": {"properties": {"target": {"const": "mp_voting_record"}}}, "then": {"required": ["mpId"]}},
                {"if": {"properties": {"target": {"const": "constituency"}}}, "then": {"required": ["postcode"]}}
            ],
            "additionalProperties": false
        }),
        Some(json!({
            "description": "Structured Parliament records returned by fetch helpers.",
            "oneOf": [
                {"type": "object"},
                {"type": "array"},
                {"type": "string"},
                {"type": "null"}
            ]
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "parliament.fetch_core_dataset",
        "Parliament: Fetch core dataset",
        "Fetch data from UK Parliament core datasets (legacy Linked Data API) and the Members API.",
        json!({
            "type": "object",
            "required": ["dataset"],
            "properties": {
                "dataset": {"type": "string"},
                "searchTerm": {"type": "string"},
                "page": {"type": "integer", "minimum": 0},
                "perPage": {"type": "integer", "minimum": 1, "maximum": 100},
                "enableCache": {"type": "boolean"},
                "fuzzyMatch": {"type": "boolean"},
                "applyRelevance": {"type": "boolean"},
                "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "description": "Raw dataset response from Parliament APIs.",
            "oneOf": [
                {"type": "object"},
                {"type": "array"}
            ]
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "parliament.fetch_bills",
        "Parliament: Fetch bills",
        "Search for UK Parliament bills via the versioned bills-api.parliament.uk service.",
        json!({
            "type": "object",
            "properties": {
                "searchTerm": {"type": "string"},
                "house": {"type": "string", "enum": ["commons", "lords"]},
                "session": {"type": "string"},
                "parliamentNumber": {"type": "integer", "minimum": 1},
                "enableCache": {"type": "boolean"},
                "applyRelevance": {"type": "boolean"},
                "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "description": "Raw JSON payload returned by the bills service.",
            "type": "object"
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "parliament.fetch_legislation",
        "Parliament: Fetch legislation",
        "Retrieve legislation metadata from legislation.gov.uk Atom feeds.",
        json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "year": {"type": "integer", "minimum": 1800},
                "type": {"type": "string", "enum": ["all", "ukpga", "ukci", "ukla", "nisi"]},
                "enableCache": {"type": "boolean"},
                "applyRelevance": {"type": "boolean"},
                "relevanceThreshold": {"type": "number", "minimum": 0.0, "maximum": 1.0}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "description": "Structured summary of legislation feed entries.",
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "year": {"type": ["integer", "string", "null"]},
                    "type": {"type": ["string", "null"]},
                    "uri": {"type": ["string", "null"], "format": "uri"},
                    "summary": {"type": ["string", "null"]}
                },
                "required": ["title"]
            }
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "parliament.fetch_mp_activity",
        "Parliament: Fetch MP activity",
        "List recent activity (debates, questions, statements) for a given MP.",
        json!({
            "type": "object",
            "required": ["mpId"],
            "properties": {
                "mpId": {"type": "integer", "minimum": 1},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50},
                "enableCache": {"type": "boolean"}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "date": {"type": "string"},
                    "type": {"type": "string"},
                    "title": {"type": "string"},
                    "description": {"type": "string"},
                    "url": {"type": ["string", "null"], "format": "uri"}
                },
                "required": ["id", "date", "type", "title", "description"]
            }
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "parliament.fetch_mp_voting_record",
        "Parliament: Fetch MP voting record",
        "Summarise an MP's voting record, optionally filtering by date range or bill.",
        json!({
            "type": "object",
            "required": ["mpId"],
            "properties": {
                "mpId": {"type": "integer", "minimum": 1},
                "fromDate": {"type": "string", "format": "date"},
                "toDate": {"type": "string", "format": "date"},
                "billId": {"type": "string"},
                "limit": {"type": "integer", "minimum": 1, "maximum": 100},
                "enableCache": {"type": "boolean"}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "divisionId": {"type": ["string", "null"]},
                    "title": {"type": ["string", "null"]},
                    "date": {"type": ["string", "null"]},
                    "vote": {"type": ["string", "null"]},
                    "majority": {"type": ["string", "null"]}
                }
            }
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "parliament.lookup_constituency_offline",
        "Parliament: Lookup constituency (offline)",
        "Resolve a postcode to its Westminster constituency using the bundled dataset.",
        json!({
            "type": "object",
            "required": ["postcode"],
            "properties": {
                "postcode": {"type": "string", "minLength": 2},
                "enableCache": {"type": "boolean"}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "type": "object",
            "properties": {
                "constituencyCode": {"type": ["string", "null"]},
                "constituencyName": {"type": ["string", "null"]},
                "mpId": {"type": ["integer", "null"]},
                "mpName": {"type": ["string", "null"]}
            }
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "parliament.search_uk_law",
        "Parliament: Search UK law",
        "Search the complete UK legislation corpus for laws, acts, and statutory instruments.",
        json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": {"type": "string", "minLength": 1},
                "legislationType": {"type": "string", "enum": ["primary", "secondary", "all"]},
                "limit": {"type": "integer", "minimum": 1, "maximum": 50},
                "enableCache": {"type": "boolean"}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "year": {"type": ["string", "null"]},
                    "legislationType": {"type": "string"},
                    "isInForce": {"type": "boolean"},
                    "url": {"type": "string", "format": "uri"},
                    "summary": {"type": ["string", "null"]},
                    "lastUpdated": {"type": ["string", "null"]}
                },
                "required": ["title", "legislationType", "isInForce", "url"]
            }
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "research.run",
        "Research: Run parliamentary research",
        "Aggregate bills, debates, legislation, votes and party balance for a parliamentary topic.",
        json!({
            "type": "object",
            "required": ["topic"],
            "properties": {
                "topic": {"type": "string", "minLength": 1},
                "billKeywords": {"type": "array", "items": {"type": "string"}},
                "debateKeywords": {"type": "array", "items": {"type": "string"}},
                "mpId": {"type": "integer", "minimum": 1},
                "includeStateOfParties": {"type": "boolean"},
                "limit": {"type": "integer", "minimum": 1, "maximum": 10}
            },
            "additionalProperties": false
        }),
        Some(json!({
            "type": "object",
            "properties": {
                "summary": {"type": "string"},
                "bills": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string"},
                            "stage": {"type": ["string", "null"]},
                            "lastUpdate": {"type": ["string", "null"]},
                            "link": {"type": ["string", "null"], "format": "uri"}
                        },
                        "required": ["title"]
                    }
                },
                "debates": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string"},
                            "house": {"type": ["string", "null"]},
                            "date": {"type": ["string", "null"]},
                            "link": {"type": ["string", "null"], "format": "uri"},
                            "highlight": {"type": ["string", "null"]}
                        },
                        "required": ["title"]
                    }
                },
                "legislation": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string"},
                            "year": {"type": ["string", "null"]},
                            "type": {"type": ["string", "null"]},
                            "uri": {"type": ["string", "null"], "format": "uri"}
                        },
                        "required": ["title"]
                    }
                },
                "votes": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "divisionNumber": {"type": ["string", "null"]},
                            "title": {"type": "string"},
                            "date": {"type": ["string", "null"]},
                            "ayes": {"type": ["integer", "null"]},
                            "noes": {"type": ["integer", "null"]},
                            "result": {"type": ["string", "null"]},
                            "link": {"type": ["string", "null"], "format": "uri"}
                        },
                        "required": ["title"]
                    }
                },
                "mpSpeeches": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "memberName": {"type": ["string", "null"]},
                            "date": {"type": ["string", "null"]},
                            "excerpt": {"type": ["string", "null"]},
                            "source": {"type": ["string", "null"], "format": "uri"}
                        }
                    }
                },
                "stateOfParties": {
                    "type": ["object", "null"],
                    "properties": {
                        "totalSeats": {"type": ["integer", "null"]},
                        "lastUpdated": {"type": ["string", "null"]},
                        "parties": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "seats": {"type": ["integer", "null"]}
                                },
                                "required": ["name"]
                            }
                        }
                    }
                },
                "advisories": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "cached": {"type": "boolean"}
            },
            "required": ["summary", "bills", "debates", "legislation", "votes", "mpSpeeches", "advisories", "cached"]
        })),
    );

    push_tool(
        &mut definitions,
        &mut input_schemas,
        "utilities.current_datetime",
        "Utilities: Current datetime",
        "Return the current UTC time alongside Europe/London local time.",
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
        Some(json!({
            "type": "object",
            "properties": {
                "utc": {"type": "string"},
                "local": {"type": "string"}
            },
            "required": ["utc", "local"]
        })),
    );

    (definitions, input_schemas)
}

fn push_tool(
    definitions: &mut Vec<ToolDefinition>,
    input_schemas: &mut HashMap<String, Value>,
    name: &str,
    title: &str,
    description: &str,
    input_schema: Value,
    output_schema: Option<Value>,
) {
    input_schemas.insert(name.to_string(), input_schema.clone());
    definitions.push(ToolDefinition {
        name: name.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        input_schema,
        output_schema,
    });
}
