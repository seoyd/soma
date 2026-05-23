You are operating as a gstack-style engineering team for Soma Zero.

Current state:
Sprint 146 is complete.

Current working core:
- Program routes data.
- AI members judge.
- Risk Governor remains final.
- Three-member pilot exists:
  - TrendEntryAI
  - RiskGuardAI
  - EvidenceRegimeAI.
- Mamba3 + Gated DeltaNet deferred member core contract exists.
- Offline batch cycle exists.
- Autonomous paper loop exists.
- Owner attention inbox exists.
- Watchlist recheck exists.
- Owner daily brief exists.
- Committee state snapshot export exists.
- Rust-only owner-console-view exists.
- Rust-only owner-console-action exists.
- owner-console-apply-actions exists.
- owner-say exists.
- OwnerNaturalInput exists.
- Local fixture news intake exists.
- AIResearchPacketBatch exists.
- build_ai_research_packets() exists.
- No web UI exists.
- No JS/TS/Tauri/Svelte exists.
- No broker/order/account/live trading path exists.
- No training/live inference exists.
- cargo test --workspace --no-run --quiet passes.
- cargo test --workspace --quiet passes.
- Acceptance is based on explicit manifest target set.

Owner correction:
Owner should not write JSON.
Owner may occasionally write one natural-language comment.
Most operation should be automatic.
News should be collected by the system.
The program should collect/normalize/route news.
AI members should analyze and judge.

Current limitation:
Sprint 146 uses deterministic keyword parsing and local fixture news intake.
That is acceptable as an early bridge, but the parser/safety keywords should not keep growing as hardcoded if/else logic.
The next step is to move intent and safety matching into a policy table / lexicon, and add a safe RSS/headline provider contract.

Sprint 147 objective:
Refactor natural owner input parsing into an explicit policy table, add a safe Rust-only news source provider layer for local/RSS/headline sources, and add a research run scheduler that periodically builds research packets for AI members.

This sprint must:
1. Keep owner UX as natural language.
2. Keep JSON as internal format only.
3. Move intent/safety keywords into configurable policy tables.
4. Add news source provider abstraction.
5. Support local fixture provider immediately.
6. Support RSS/headline provider contract behind explicit config.
7. Avoid browser/JS scraping.
8. Avoid full article copying.
9. Route collected news into AI research packets.
10. Keep AI members as judgment source.
11. Keep everything paper-only.

────────────────────────────────────────
0. SPRINT NAME
────────────────────────────────────────

gstack Sprint 147:
Owner Intent Policy Table + Safe RSS/Headline News Provider + Research Run Scheduler

────────────────────────────────────────
1. HARD RULES

Do not add:
- web UI
- Tauri
- Svelte
- React
- JavaScript
- TypeScript
- browser server
- dashboard server
- browser scraping
- paywall/login/private scraping
- full article copying
- broker/order/account
- live trading
- order execution
- model training
- live inference
- runtime LLM debate
- real Mamba3 runtime
- real Gated DeltaNet runtime
- central MoE committee model
- report bloat
- broad test suite

Do not:
- require owner to write JSON
- expose internal action JSON as primary owner UX
- let owner text become order
- let news text become guaranteed truth
- let program produce buy/sell decision directly
- bypass AI members
- bypass Risk Governor
- mutate model weights
- claim real AI inference
- rely on work.md
- touch Group B
- claim old broad auto-discovery acceptance

Allowed:
- Rust-only policy table for owner intent parsing
- Rust-only safety lexicon
- local JSON/TOML policy fixture
- news provider trait
- local news fixture provider
- RSS/headline provider contract behind config
- no network calls in tests
- optional live network disabled by default
- research run scheduler for local/offline mode
- existing CLI extension
- small focused tests only

Main rule:
Program collects/routes/schedules.
AI members judge.
Risk Governor remains final.
Everything remains paper-only.

────────────────────────────────────────
2. FEATURE A — OWNER INTENT POLICY TABLE

Problem:
Current tests use example hardcoded phrases like:
- “리스크와 변동성이 걱정돼”
- “관심종목으로 지켜봐”
- “위원회 다시 재검토해줘”
- “계좌에서 주문해”
- “레버리지 최대로”
- “수익 보장”

This is okay for tests, but parser policy should be centralized.

A. OwnerIntentPolicy

Fields:
- policy_id
- language:
  - Ko
  - En
  - Mixed
- intent_rules: Vec<OwnerIntentRule>
- safety_rules: Vec<OwnerSafetyRule>
- default_intent:
  - Unknown
  - Comment
- paper_only: true

B. OwnerIntentRule

Fields:
- rule_id
- intent:
  - Comment
  - RiskConcern
  - EvidenceRequest
  - WatchlistRequest
  - ReconsiderationRequest
  - PaperOutcomeLabel
  - Unknown
- include_terms
- exclude_terms
- priority
- confidence_hint

C. OwnerSafetyRule

Fields:
- rule_id
- blocked_category:
  - OrderExecution
  - BrokerAccount
  - LiveTrading
  - Leverage
  - GuaranteedReturn
  - PrivateIllegalInfo
  - SecretCredential
- blocked_terms
- severity:
  - Reject
  - Warn
- rejection_message

D. OwnerIntentPolicyLoadResult

Fields:
- loaded
- rule_count
- safety_rule_count
- warnings
- policy

E. load_owner_intent_policy_from_local_file(path)

Rules:
- local path only
- reject remote path
- reject traversal
- deterministic ordering
- if absent, use built-in default policy

F. parse_owner_natural_input_with_policy(input, policy)

Rules:
- use policy table
- no scattered hardcoded keyword matching
- higher priority rule wins
- safety rules run before intent rules
- unsafe input rejected
- unknown input becomes Unknown or Comment based on policy
- paper_only always true

G. Built-in default Korean policy

Must include at least:
EvidenceRequest terms:
- 근거
- 증거
- 뉴스 부족
- 다시 확인

RiskConcern terms:
- 리스크
- 위험
- 변동성

WatchlistRequest terms:
- 관심종목
- 지켜봐
- watch

ReconsiderationRequest terms:
- 다시 봐
- 재검토
- 위원회 다시

PaperOutcomeLabel terms:
- paper positive
- 결과 좋음
- 성과 좋음

Comment terms:
- 확인
- 메모

Safety blocked terms:
- 주문
- 실거래
- 계좌
- 브로커
- 레버리지
- 최대 매수
- 수익 보장
- API key
- secret
- private info
- illegal info

Important:
Tests can still use fixed sample phrases.
Runtime parser should use policy table.

────────────────────────────────────────
3. FEATURE B — SAFE NEWS SOURCE PROVIDER

A. NewsProviderKind

Enum:
- LocalFixture
- RssFeed
- HttpHeadline
- Disabled

B. NewsProviderConfig

Fields:
- provider_id
- kind
- enabled
- source_path_or_url
- source_label
- allowed_domains
- symbols
- market_scopes
- max_items
- timeout_ms
- trust_level:
  - High
  - Medium
  - Low
  - ReviewRequired

Rules:
- LocalFixture is default.
- RssFeed/HttpHeadline disabled by default.
- network provider allowed only if explicitly enabled.
- no network in tests.
- allowed_domains checked by host, not substring.
- no browser/JS.
- no paywall/login/private content.
- no full article body.
- headline/summary/source/timestamp only.

C. NewsProvider trait

trait NewsProvider {
    fn collect(&self, config: &NewsProviderConfig) -> NewsCollectionResult;
}

Implement:
- LocalFixtureNewsProvider
- RssFeedNewsProviderStub
- HttpHeadlineNewsProviderStub

For Sprint 147:
- LocalFixtureNewsProvider works.
- RssFeed/HttpHeadline can return Disabled/Deferred unless explicitly enabled.
- If enabled but no safe fetch implementation exists, return safe “ProviderDeferred” status.

Do not implement heavy network fetch yet unless already available without new dependency.

D. NewsCollectionRun

Fields:
- run_id
- provider_results
- collected_news
- rejected_news
- safety_summary

E. collect_news_from_providers(configs)

Flow:
1. Validate providers.
2. Run enabled local providers.
3. For RSS/HTTP, enforce allowlist and disabled-by-default.
4. Normalize into CollectedNewsItem.
5. Convert into NewsSnapshot.
6. Return NewsCollectionRun.

────────────────────────────────────────
4. FEATURE C — RESEARCH RUN SCHEDULER

A. ResearchRunConfig

Fields:
- research_run_id
- market_scopes
- symbols
- market_data_path optional
- news_provider_config_path optional
- owner_intent_policy_path optional
- owner_comment_text optional
- owner_comment_path optional
- member_state_input_path optional
- offline_member_output_batch_path optional
- run_mode:
  - SingleShot
  - ManualStep
  - FixedCount
- max_cycles
- paper_only: true

B. ResearchRunResult

Fields:
- research_run_id
- news_collection_run
- research_packet_batch
- member_opinion_count
- event_count
- committee_session_count
- owner_feedback_generated_count
- safety_summary

C. run_research_packet_pipeline(config)

Flow:
1. Load market data.
2. Load or build owner intent policy.
3. Parse owner comment text/path if provided.
4. Collect news via providers.
5. Build AIResearchPacketBatch.
6. Feed packets into existing batch committee cycle if configured.
7. Return ResearchRunResult.
8. Do not place orders.
9. Do not run live inference.

D. Research packet summary

Expose:
- symbols covered
- news items attached
- owner context attached
- members routed
- packets generated
- events generated

This is not a trading signal.

────────────────────────────────────────
5. CLI

Reuse existing commands where possible.

A. owner-say

Update:
soma-experiment owner-say \
  --text "삼성전자 뉴스 근거가 부족해 보여. 다시 봐." \
  --symbol 005930.KS \
  --scope KoreaShortTerm \
  --policy examples/owner_intent_policy.ko.sample.json \
  --out target/minimal_committee_state/owner_attention_actions.json

Rules:
- policy optional.
- if absent, built-in default policy used.

B. minimal-ai-committee-cycle

Add optional config:
- owner_intent_policy_path
- news_provider_config_path
- research_run_enabled
- emit_research_run_summary

C. Optional small command if necessary:
soma-experiment research-run --config examples/soma_minimal_ai_committee_core.toml

Prefer not adding this unless needed.

Do not add a CLI family explosion.

────────────────────────────────────────
6. FILE SCOPE

Prefer changing only:
- src/league/minimal_ai_committee_core.rs
- src/bin/soma_experiment.rs
- tests/minimal_ai_committee_core.rs
- examples/soma_minimal_ai_committee_core.toml
- examples/owner_intent_policy.ko.sample.json
- examples/news_providers.sample.json
- examples/minimal_news_items.sample.json
- optional docs/SPRINT147_POLICY_NEWS_RESEARCH_PIPELINE.md

Do not create many files.
Do not add JS/TS/Tauri/Svelte.
Do not add web assets.

────────────────────────────────────────
7. TESTS

Add focused tests inside tests/minimal_ai_committee_core.rs.

Required tests:
1. built-in owner intent policy parses EvidenceRequest.
2. built-in owner intent policy parses RiskConcern.
3. built-in owner intent policy parses WatchlistRequest.
4. built-in owner intent policy parses ReconsiderationRequest.
5. built-in policy rejects order/account/leverage/guaranteed-return text.
6. local owner intent policy file loads.
7. owner policy remote path is rejected.
8. policy priority works.
9. tests may use fixed sample phrases, but parser uses policy table.
10. LocalFixtureNewsProvider loads news.
11. RSS provider disabled by default.
12. disallowed remote domain rejected by host.
13. full article body is not stored.
14. collected news converts to NewsSnapshot.
15. research packets include news by symbol/scope.
16. research packets include owner context.
17. DataRouter still only routes.
18. research run does not generate buy/sell directly.
19. AI members remain opinion source.
20. Risk Governor remains final.
21. no broker/order/account path exists.
22. no training/live inference path exists.
23. no browser/JS/Tauri/Svelte dependency exists.
24. deterministic repeated research run.

Do not add broad test files.

────────────────────────────────────────
8. ACCEPTANCE CRITERIA

Sprint 147 succeeds if:

- OwnerIntentPolicy exists.
- hardcoded runtime keyword parsing is replaced by policy-table parsing.
- tests still use sample phrases safely.
- owner-say can use built-in or local policy.
- unsafe owner text is rejected by policy.
- NewsProvider abstraction exists.
- LocalFixtureNewsProvider works.
- RSS/HTTP providers are disabled/deferred safely by default.
- news converts to NewsSnapshot.
- AIResearchPacketBatch uses collected news.
- research run pipeline exists.
- program still routes only.
- AI members still judge.
- Risk Governor remains final.
- no broker/order/account path exists.
- no live trading path exists.
- no model training/live inference is added.
- no real Mamba/Gated runtime is added.
- no web/browser/JS/Tauri/Svelte is added.
- focused tests pass.
- explicit manifest workspace tests pass.

────────────────────────────────────────
9. RUN COMMANDS

Run:
cargo fmt --all
cargo check --workspace
cargo build --bin soma_experiment
cargo test --test minimal_ai_committee_core --quiet
cargo test --test workspace_timeout_reduction_queue --quiet

Owner-say smoke:
cargo run --quiet --bin soma_experiment -- owner-say \
  --text "삼성전자 뉴스 근거가 부족해 보여. 다시 봐." \
  --symbol 005930.KS \
  --scope KoreaShortTerm \
  --out target/minimal_committee_state/owner_attention_actions.json

Cycle/research smoke:
cargo run --quiet --bin soma_experiment -- minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml

Workspace:
cargo test --workspace --no-run --quiet
cargo test --workspace --quiet

────────────────────────────────────────
10. FINAL RESPONSE FORMAT

Keep short:

## 1. What changed

## 2. Owner intent policy table

## 3. Natural owner input

## 4. News provider layer

## 5. Research packet pipeline

## 6. CLI usage

## 7. Safety preserved

## 8. Files changed

## 9. Tests run

## 10. Workspace status

## 11. Still deferred

## 12. Next step

No giant report.
No 60-section output.