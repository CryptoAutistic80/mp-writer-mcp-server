Technical Plan for Enhancing MCP Server Research Capabilities
=============================================================

1. Context and Objectives
-------------------------
The MP-writer MCP server currently provides AI-generated letters for constituents to send to MPs. To set a new standard of excellence, the server must deliver accurate, up-to-date research on UK parliamentary activities—laws, debates, bills and voting records—so that letters are persuasive and evidence-driven. The free-to-use APIs identified previously (Members/Commons votes/Bills/Hansard, Legislation.gov.uk, TheyWorkForYou, etc.) provide the necessary data[1].

Key objectives:

- Integrate official UK parliamentary APIs (Members, Bills, Commons votes, Hansard) and the Legislation.gov.uk API to search legislation, bills and debates[1]. Use TheyWorkForYou only for features not covered by the official APIs (e.g., MP speech look-ups) and avoid postcode lookup (the app already knows the MP).
- Design robust, modular Rust code that follows the project’s module structure rules (DTOs in dto.rs, methods in impl.rs, handlers in handler.rs, helpers in helpers.rs), uses a single Sled database instance, and avoids .unwrap() outside tests.
- Implement rate-limited HTTP clients, caching and summarisation to return concise, reliable research segments that can be included in letters.

2. Assumptions About Existing Repository
----------------------------------------
Because the GitHub connector is unavailable, the existing codebase could not be examined directly. The plan therefore makes reasonable assumptions about typical MCP server architecture:

- The server is written in Rust (likely using tokio/reqwest and axum/warp for HTTP) and is structured into modules with dto.rs, impl.rs, handler.rs and helpers.rs as described in the development rules.
- There is an existing research or knowledge service that queries limited sources (e.g., generic web search); this needs to be extended.
- The server stores context or search results in a Sled database for caching. Only one open_tree() call is made in main.rs and the database handle is injected into services.
- There is already an endpoint for generating letters using AI; we will add research endpoints or integrate research into this workflow.

3. Proposed Architecture Enhancements
-------------------------------------

3.1 API Integration Layer
~~~~~~~~~~~~~~~~~~~~~~~~~~
Implement a dedicated module for external data retrieval. This layer will encapsulate all HTTP calls and handle authentication, rate limits and caching. Recommended structure (under src/research/parliament/):

- dto.rs – Define data transfer objects for each API:
  - MemberDto, VoteDto, BillDto, DebateDto for UK Parliament data.
  - LegislationDto for legislation metadata/content.
  - SpeechDto for MP speeches (if using TheyWorkForYou).
  - Use serde for JSON deserialisation and keep them data-only.
- impl.rs – Implement asynchronous functions to fetch data:
  - async fn fetch_current_house_composition() -> Result<StateOfPartiesDto> — calls the “State of the Parties” endpoint from commonsvotes-api to determine party composition[1].
  - async fn fetch_bills(keyword: &str) -> Result<Vec<BillDto>> — searches bills-api.parliament.uk for bills matching the user’s topic.
  - async fn fetch_debates(keyword: &str) -> Result<Vec<DebateDto>> — queries Hansard for debates matching the topic (historic and recent). Include date filters as optional parameters.
  - async fn fetch_legislation(act_id: &str) -> Result<LegislationDto> — retrieves full Acts or sections from Legislation.gov.uk[1].
  - async fn fetch_mp_speeches(member_id: u32, keyword: &str) -> Result<Vec<SpeechDto>> — optional: call TheyWorkForYou to fetch a specific MP’s statements on the topic (requires API key but free for low-volume use).

Each function should:

- Use reqwest::Client with a rate-limiter (e.g., tower::limit or custom token-bucket) to stay within free usage limits.
- Parse JSON into DTOs, mapping only the fields needed for research.
- Log requests and handle HTTP errors gracefully (use anyhow or custom error types); never call .unwrap().

Cache results using the Sled DB (see §3.3).

helpers.rs – Provide helper functions for constructing URLs, handling query parameters, and optionally converting HTML/markup to plain text. For example, convert Hansard transcripts to plain text and summarise them.

3.2 Service Layer
~~~~~~~~~~~~~~~~~
Create a research service that coordinates multiple API calls and returns aggregated research results. Structure:

- research/dto.rs – Define a ResearchRequestDto (fields: topic: String, mp_id: Option<u32>, bill_keywords: Vec<String>, debate_keywords: Vec<String> etc.) and ResearchResponseDto (fields: summary: String, bills: Vec<BillDto>, debates: Vec<DebateDto>, laws: Vec<LegislationDto>, votes: Vec<VoteDto>, mp_speeches: Vec<SpeechDto>, state_of_parties: Option<StateOfPartiesDto>).
- research/impl.rs – Implement async fn run_research(req: ResearchRequestDto, db: &Db) -> Result<ResearchResponseDto> that:
  - Checks the Sled cache for existing results keyed by topic + date; if present and fresh (e.g., <7 days), return cached result.
  - Uses the API integration layer to fetch bills, debates, legislation, votes and MP speeches. Execute calls concurrently via join! to improve latency.
  - Summarises long transcripts or bill descriptions using an internal summarisation function (may call the AI engine or summarise_text helper). Ensure summarisation length fits within prompt budgets.
  - Assembles a narrative summary that emphasises key facts, cites acts, quotes relevant debates, and highlights the MP’s voting record. Provide citations with URLs or API references that can be included in the letter.

Stores the assembled ResearchResponseDto in Sled for future reuse.

- research/handler.rs – Expose an MCP tool endpoint, e.g., /research/execute, that accepts ResearchRequestDto and returns ResearchResponseDto as JSON. Ensure the endpoint:
  - Validates input (e.g., non-empty topic).
  - Returns appropriate HTTP status codes.
  - Is documented in openapi.yaml (if you provide OpenAPI specs).

3.3 Caching and Database Usage
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Follow the rule of a single Sled DB instance:

- In main.rs, open the Sled database and create named trees for each API (members, bills, debates, etc.) only once. Pass Arc<Db> handles into services.
- Write helper functions in helpers.rs to get/set values in each tree with TTL (store timestamp alongside JSON). Use composite keys (e.g., bills:{keyword}) and serialise DTOs via serde_json.
- Optionally implement a periodic clean-up task (e.g., using tokio::spawn(async move { ... })) to purge stale entries.

3.4 Summarisation and Text Processing
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Large transcripts and legislation sections need summarisation so the letter remains concise. Add a summariser module:

- Implement async fn summarise_text(text: &str, max_words: usize) -> String. If you have an AI model accessible internally, call it; otherwise implement a simple extractive summariser (e.g., text_rank) or rely on the user’s AI pipeline.
- Use this summariser in the research service to produce digestible synopses of debates, bill explanatory notes, or long laws. Provide bullet points of key arguments or provisions.

3.5 Configuration and Secrets
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Use environment variables (via dotenvy or std::env) for API keys (e.g., TWFY_API_KEY), base URLs, and rate-limit parameters. Create a config.rs module to load and expose them.
- Document the need to set the TWFY_API_KEY if TheyWorkForYou features are enabled. For other APIs like bills-api and legislation.gov.uk, no key is needed[1].

3.6 Error Handling and Testing
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Create a unified error.rs with custom error types (e.g., ApiError, CacheError), implement From<reqwest::Error>, and use thiserror crate. Return Result<T, Error> from all functions.
- Add integration tests in tests/research_tests.rs that:
  - Mock HTTP responses using wiremock or mockito to simulate API endpoints.
  - Verify that DTOs deserialise correctly and that the run_research function aggregates and summarises data as expected.
  - Test caching logic by ensuring repeat calls return cached results.

3.7 Updating Existing Code
~~~~~~~~~~~~~~~~~~~~~~~~~~
To integrate these features into the existing server:

1. Refactor existing structs into dto.rs and impl.rs if not already in place. Move any data-only structs out of functions and remove unwrap() calls; use match/if let patterns for safe handling.
2. Add new modules (src/research/parliament, src/research) and update mod.rs files to pub use exported structs and functions.
3. Update main.rs:
4. Open Sled database and pass handles to research service.
5. Register the new research endpoint with the web framework.
6. Load configuration from environment.
7. Add CLI/tool metadata for the new research function so that the MCP client knows how to call it (update mcp.json or equivalent).
8. Document API usage in the repository README.md and provide examples of the new endpoint in action.

3.8 Future Enhancements
~~~~~~~~~~~~~~~~~~~~~~~

- Topic Suggestion: Provide auto-suggested related topics for constituents (e.g., similar bills or debates). Could be implemented by analysing the returned data or using additional open datasets.
- Sentiment and Bias Analysis: Use natural language processing to assess the tone of debates or laws and present balanced perspectives.
- User Feedback Loop: Allow users to rate research sections so the system learns which sources are most helpful.

4. Conclusion
-------------
The plan above introduces a comprehensive, modular research subsystem for the MP-writer MCP server. By leveraging official UK Parliament APIs and legislation datasets[1], caching results in a single Sled database, and following strict Rust module conventions, the server will deliver accurate, timely research to users. Summarisation ensures the output remains digestible, while proper error handling, testing and documentation guarantee maintainability. Implementing these enhancements will enable constituents to craft well-informed letters backed by authoritative data on bills, debates, votes and laws.

[1] Developer hub - UK Parliament
https://developer.parliament.uk/
