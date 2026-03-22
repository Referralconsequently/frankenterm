# Changelog

All notable changes to FrankenTerm (`ft`) are documented in this file.

Organized by landed capabilities, not raw diff order. Each version section describes what shipped and why it matters. Commit links point to representative commits for each capability area.

- **Repository**: <https://github.com/Dicklesworthstone/frankenterm>
- **Default branch**: `main`
- **Tags**: listed under [Tags & Releases](#tags--releases)

---

## [Unreleased] — development on `main` since 2026-02-17

> ~3,450 commits since the `backup-before-rewrite` tag. Active daily development by concurrent agent swarms. The project has grown from a WezTerm automation wrapper to a full terminal platform with its own GUI, mux server, and 120-crate workspace.

### WezTerm Source Import & FrankenTerm Identity (2026-02-10)

Imported WezTerm source at commit `05343b38` and integrated it as owned code within the workspace. Renamed the project from `wezterm_automata`/`wa` to `frankenterm`/`ft`. All CLI commands, module names, config paths, and documentation updated.

- [Import WezTerm source as FrankenTerm owned code](https://github.com/Dicklesworthstone/frankenterm/commit/e6303733ef911cf7eae8e6c0569a963049315f8c)
- [Integrate FrankenTerm crates into workspace](https://github.com/Dicklesworthstone/frankenterm/commit/09cf50f95fe9524b56b90dfe97fe70d9638a3c56)
- [rename: wezterm_automata/wa -> frankenterm/ft](https://github.com/Dicklesworthstone/frankenterm/commit/4303a0a32806963d4da1044515c11c416c25e812)
- [Complete wa->ft naming migration in CLI](https://github.com/Dicklesworthstone/frankenterm/commit/bf83c4bc576b5a8072500f153411b337947ca3cf)

### Native GUI Terminal (2026-03-02)

Added `frankenterm-gui` crate: a working terminal window that opens natively on macOS, bundled as `FrankenTerm.app` with vendored font rendering. Integrated native event bridge for `ft watch` push-mode observation. TOML-first config with Lua opt-in.

- [Add frankenterm-gui crate to workspace](https://github.com/Dicklesworthstone/frankenterm/commit/07544966c5f7510b885db6b67db908750bafc497)
- [Working terminal window opens on macOS](https://github.com/Dicklesworthstone/frankenterm/commit/7a61af81f439881a9e38fd206aa1a56902e422e0)
- [Build FrankenTerm.app from source, no WezTerm dependency](https://github.com/Dicklesworthstone/frankenterm/commit/8610c04e5d64ccb0d22b36104708c46528c962d1)
- [Add native event bridge emitter for ft watch integration](https://github.com/Dicklesworthstone/frankenterm/commit/35949cd742f564321c56c9498abb19666fc3fe4d)
- [TOML-first config with Lua opt-in and FrankenTerm paths](https://github.com/Dicklesworthstone/frankenterm/commit/4460f6db417749809a2aad0131d97d351bcab98c)

### Mux Server & Workspace Vendoring (2026-03-02)

Vendored GUI layer crates and mux server into the workspace. Added swap layouts, floating pane toggle, stack cycling, and SSH domain config.

- [Add frankenterm-mux-server binary and library](https://github.com/Dicklesworthstone/frankenterm/commit/81ec52a09dd3c64382779f30eb705de1bbc4b1f6)
- [Vendor GUI layer crates into workspace](https://github.com/Dicklesworthstone/frankenterm/commit/dfd875e7df23ad1802e1dba9d3786bdd52860936)
- [Add swap layouts, floating pane toggle, and stack cycling actions](https://github.com/Dicklesworthstone/frankenterm/commit/2d422a37aa4a73856e7894b658ca5990702aece6)
- [Agent-aware session management with state detection](https://github.com/Dicklesworthstone/frankenterm/commit/1ea89d500d8558be0c5ae1875803a2135f5fbbe2)
- [Integrated swarm dashboard panel](https://github.com/Dicklesworthstone/frankenterm/commit/74febab420a447477dc715f7929f37ad06a60880)

### Swarm Orchestration Runtime (2026-03-03)

Purpose-built fleet management for 200+ concurrent AI agents: deterministic launch plans with phased startup and weighted ordering, dependency-aware work queues with anti-starvation fairness, Agent Mail coordination kernel, and swarm pipeline for DAG-ordered orchestration.

- [Deterministic fleet launch plan with phased startup](https://github.com/Dicklesworthstone/frankenterm/commit/3052b1822f6f5ecff098cde452abb4cd3a98acaa)
- [Dependency-aware swarm work queue with anti-starvation](https://github.com/Dicklesworthstone/frankenterm/commit/4f68ec851970fddab642201e46910792c8a691db)
- [Mission Agent Mail coordination kernel](https://github.com/Dicklesworthstone/frankenterm/commit/9fbf9503a91d30e2426310e86589eb48a6e3f466)
- [Swarm pipeline for DAG-ordered fleet orchestration](https://github.com/Dicklesworthstone/frankenterm/commit/a94bf4fece31ab0bab4e7d715a408ee34d9e17bf)
- [Swarm command center dashboard and command palette](https://github.com/Dicklesworthstone/frankenterm/commit/8acd9ae7f57fd75e35fcf420175263c42052bd4d)

### Native Mux Lifecycle (2026-03-02 -- 2026-03-03)

Ground-up native mux subsystem: lifecycle state machine with concurrency control, command transport primitives, topology orchestration, session profiles/templates, durable state checkpoints with rollback, headless/federated mux server for remote fleet control, and connector host runtime.

- [Native mux lifecycle state machine](https://github.com/Dicklesworthstone/frankenterm/commit/ab8928de2170cf2b7d938fd65b3abe69dec2fb34)
- [Command transport primitives](https://github.com/Dicklesworthstone/frankenterm/commit/1386cfbfd6265d44f7c165e6b8cdd6196d6c8ba6)
- [Topology orchestration service](https://github.com/Dicklesworthstone/frankenterm/commit/16d2ebcf2cde252704d0b618922f2308fac5f676)
- [Session profile/template/persona engine](https://github.com/Dicklesworthstone/frankenterm/commit/d282b89e6fd12b732f65742491841ae925f8659d)
- [Durable state checkpoint/rollback subsystem](https://github.com/Dicklesworthstone/frankenterm/commit/e0733bea4089db2432c175f1834098f133d41405)
- [Headless/federated mux server for remote fleet control](https://github.com/Dicklesworthstone/frankenterm/commit/228ba05fdf1b08b256195c5b6c972d02786913b5)

### Connector SDK & Extension System (2026-03-03)

Multi-host connector mesh federation, connector SDK with builders/linting/certification/simulator, inbound/outbound bridges, and canonical event schema with evolution tooling.

- [Multi-host connector mesh federation](https://github.com/Dicklesworthstone/frankenterm/commit/5c1993892c8d112183ad62d1ff120c6805ccc89f)
- [Connector SDK devkit with certification pipeline](https://github.com/Dicklesworthstone/frankenterm/commit/67eee8318df9d5da1976c5aaadd3fbb2c123c5e4)
- [Connector outbound bridge action routing](https://github.com/Dicklesworthstone/frankenterm/commit/602df4d7d52efd0dec94a84209133c1152e44713)
- [Sandbox capability envelope and zone enforcement](https://github.com/Dicklesworthstone/frankenterm/commit/6634aea6a6dc96f9d41437f100f5bee2432e8c78)

### 21-Subsystem Policy Engine (2026-03-10)

Expanded the policy engine from basic authorize() to a unified governance framework integrating 21 subsystems: quarantine registry, kill-switch, hash-linked tamper-evident audit chain, compliance reporting, credential broker, connector governor, namespace isolation, approval tracker, revocation registry, and forensic report generator.

- [Quarantine registry and kill-switch primitives](https://github.com/Dicklesworthstone/frankenterm/commit/d33b8a1d927fb15d616ed28c6be3124236b36732)
- [Hash-linked tamper-evident audit chain](https://github.com/Dicklesworthstone/frankenterm/commit/f37d2bf5bce393159d6d7b8cdcf447705da8836d)
- [Compliance reporting engine](https://github.com/Dicklesworthstone/frankenterm/commit/4f58605c3a329d696fa6e7461972f0acd9dc62cb)
- [Credential broker integration](https://github.com/Dicklesworthstone/frankenterm/commit/bcfe80193e3e7ef4473f91c07599cff00d567d56)
- [Namespace isolation for multi-tenant connectors](https://github.com/Dicklesworthstone/frankenterm/commit/144cf6bc5b4bf3a5af03ae894b1827d6e95abef1)
- [Forensic report generator with query/export pipeline](https://github.com/Dicklesworthstone/frankenterm/commit/34bcc891a5a42fdb8bbc785b947e5f21bef74c27)

### Transaction Execution Engine (2026-03-13 -- 2026-03-17)

Multi-pane transactional operations with prepare/commit/compensate lifecycle, idempotency guards, deterministic replay, mission journal with compaction, and `ft tx` CLI subcommand.

- [Tx execution engine: prepare/commit/compensate lifecycle](https://github.com/Dicklesworthstone/frankenterm/commit/f1f129300ee4b2db63a13e8159899f418bd8cece)
- [ft tx subcommand for mission transaction control](https://github.com/Dicklesworthstone/frankenterm/commit/62d3fc446bfe46745661dc2684487845c08e2d4f)
- [Crash-consistent mission journal with compaction and replay](https://github.com/Dicklesworthstone/frankenterm/commit/6fee81d1a9f16405884d24eedd5a34c7de34af43)
- [Harden resume safety, compensation step results, persist contract state](https://github.com/Dicklesworthstone/frankenterm/commit/3665ca9b2bb458c1dfa3b2b44dee080b60a60c47)
- [Mission abort-with-checkpoint, lock lease renewal, snapshot metadata](https://github.com/Dicklesworthstone/frankenterm/commit/fe3583b9528ac881d04d519cdd0119079d3dfef4)

### Tiered Scrollback & Fleet Memory Controller (2026-03-12)

Three-tier memory management (hot/warm/cold) for 200+ pane workloads. Unified fleet memory controller synthesizing backpressure from queue depth, system memory, and per-pane budgets with hysteresis.

- [Tiered scrollback storage for 200+ pane agent swarms](https://github.com/Dicklesworthstone/frankenterm/commit/ba9fc94987150304c5d506c7b2d147e3a654a9ba)
- [Fleet memory controller unifying 5 memory subsystems](https://github.com/Dicklesworthstone/frankenterm/commit/b6b93b86d6dd6bd12d780d06a92df46823ac4fb8)
- [Per-pane arena byte accounting with peak watermark](https://github.com/Dicklesworthstone/frankenterm/commit/f981a6f998ccd94fb98a6d638a3eb56e9582220f)
- [Core-level swarm stress tests for 200-pane workloads](https://github.com/Dicklesworthstone/frankenterm/commit/2f7f73c18809773a3845ae09c5ba1a95cf6a7bbf)

### Distributed Mode Hardening (2026-03-11 -- 2026-03-17)

Protocol version validation on handshake, gap cursor seeding with interleaved chronological replay, session-scope tracking with reconnect cleanup, stale scope pruning with heartbeat tracking, and session checkpoint save/restore.

- [Protocol version validation on handshake](https://github.com/Dicklesworthstone/frankenterm/commit/287efbe432d1eec71e089bc55ea8e3d760e347b4)
- [Gap cursor seeding and interleaved chronological replay](https://github.com/Dicklesworthstone/frankenterm/commit/bd9cadabe8dabf4260e938ede636e311ac2371d1)
- [Session-scope tracking with reconnect cleanup](https://github.com/Dicklesworthstone/frankenterm/commit/120e79375b0100975bce117d613245307b275a63)
- [Session checkpoint save/restore with aggregator state](https://github.com/Dicklesworthstone/frankenterm/commit/1b3d0aed30cd3400eb44b510b550049f4f72e309)
- [Harden security error responses to avoid info leakage](https://github.com/Dicklesworthstone/frankenterm/commit/e138ed36241238cb0d686320afb302ec2ade7d02)

### Replay & Forensics (2026-03-17)

Sensitivity tiers, redaction policy, and causal chain fields for replay events. Deterministic replay with canonical string methods for tx operations.

- [Add sensitivity tiers, redaction policy, and causal chain fields](https://github.com/Dicklesworthstone/frankenterm/commit/8b8ec5c6f136ee321f3376be8d000f33677cb4d2)
- [Add PartialEq/Eq derives and canonical_string methods for tx replay](https://github.com/Dicklesworthstone/frankenterm/commit/05a3a8bd79b5affca4cc693f72b103473802f3a9)

### CASS Export (2026-03-20)

New `cass-export` feature for exporting recorder sessions to CASS connectors. Includes token estimation fixes and export session resolution hardening.

- [Add cass-export feature](https://github.com/Dicklesworthstone/frankenterm/commit/a84f6fcd48d9df1ca51e04fe38f539a7a822bcab)
- [Correct token estimation for whitespace-only splits](https://github.com/Dicklesworthstone/frankenterm/commit/1cd36ab36bd2d7ac036e90018088543735fe8ea5)

### Async Runtime Migration: tokio -> asupersync (2026-02-11 -- 2026-03-11)

Systematic migration from tokio to the asupersync runtime with `runtime_compat` abstraction layer. All 111 `#[tokio::test]` tests migrated. Benchmarks migrated. Feature-gated dual-runtime compatibility surface maintained during transition.

- [asupersync-runtime flag and cx_creation bench](https://github.com/Dicklesworthstone/frankenterm/commit/8d293835b582906e701589c587d711f1ff2f1594)
- [Runtime abstraction layer for asupersync migration](https://github.com/Dicklesworthstone/frankenterm/commit/7832fd757039b33c8872afc91dfc4f7e618c1a3f)
- [Enable all features by default, migrate test suite](https://github.com/Dicklesworthstone/frankenterm/commit/b66eb6a60caa5a83d7f4bd2235350161fcc86a94)
- [Migrate all 111 #[tokio::test] to asupersync compat runtime](https://github.com/Dicklesworthstone/frankenterm/commit/2482469a876164f0ef6604fd7046cd9ba7408dfd)
- [Replace async-io and async-channel with promise and flume](https://github.com/Dicklesworthstone/frankenterm/commit/183f04dc58df37e1cd15bd34033f4c4a220f6a74)

### WASM Extension System (2026-02-13)

WASM extension sandbox with security model, module cache, host function API, FrankenTerm Extension Package Format (.ftx), extension lifecycle management, and event bus/keybinding/storage APIs.

- [WASM extension sandbox security model](https://github.com/Dicklesworthstone/frankenterm/commit/8208656979a255625cdd0853677fbc6c17d04fca)
- [FrankenTerm Extension Package Format .ftx](https://github.com/Dicklesworthstone/frankenterm/commit/a0211244a9277c126e1e98e2efa2beaf7234b70e)
- [Extension lifecycle management](https://github.com/Dicklesworthstone/frankenterm/commit/26b8540f520186e03f94045079e769bf4522d233)
- [Config migration tool: wezterm.lua to frankenterm.toml](https://github.com/Dicklesworthstone/frankenterm/commit/37e4063191b6ad2203dba1f8f05075d11775d44f)

### Session Persistence & Restore (2026-02-10)

Complete session persistence stack: pane state snapshots, topology serializer, SnapshotEngine orchestrator, layout restoration engine, session retention policy, session restore from unclean shutdowns, and process re-launch engine.

- [Session persistence: pane state snapshots and topology serializer](https://github.com/Dicklesworthstone/frankenterm/commit/038d2f1e808c84fe2f171b9c4a09ff53bff15292)
- [SnapshotEngine orchestrator](https://github.com/Dicklesworthstone/frankenterm/commit/56f0fc79758f0135767309ed6642501bdd3161e6)
- [Layout restoration engine](https://github.com/Dicklesworthstone/frankenterm/commit/72717c857b23e85bc839369b9929850ce72cbf69)
- [Session restore engine: detect and recover from unclean shutdowns](https://github.com/Dicklesworthstone/frankenterm/commit/bfd0a82edf85e6700121f302baf50ff52d2da90f)
- [Process re-launch engine](https://github.com/Dicklesworthstone/frankenterm/commit/76f7949bb716be685a2934a33582bc93fca58d42)
- [ft session CLI subcommands](https://github.com/Dicklesworthstone/frankenterm/commit/cbbfec73442b35f5d8b36bacbc33a8ce36e0b23d)

### FTUI Migration (2026-02-08 -- 2026-02-09)

Complete TUI rewrite: one-writer output routing, app shell with Model impl, canonical keybinding table, input dispatcher, command execution state machine, migrated Events/Triage/History/Search/Help views, chaos tests, runtime backend selection for phased rollout.

- [FTUI migration foundation (FTUI-01 through FTUI-04.2)](https://github.com/Dicklesworthstone/frankenterm/commit/ecb7684264f931baf489ffa1106a422f9cc2e9fb)
- [Canonical keybinding table and input dispatcher](https://github.com/Dicklesworthstone/frankenterm/commit/ef78f349f7a4ad31f2161e5c443a89bdfdae9c8e)
- [Migrate Triage view with ranked items and workflow panel](https://github.com/Dicklesworthstone/frankenterm/commit/f09a06665dc2448778d3eab90fab4d665b6d697c)
- [Runtime backend selection for phased rollout](https://github.com/Dicklesworthstone/frankenterm/commit/f1a09e1a087462807398eaabcb5d86ae4adcddb5)
- [Close FTUI-08 perf hardening + master FTUI epic](https://github.com/Dicklesworthstone/frankenterm/commit/feb018d3c569b95b0e2aee80facf3189113890ae)

### Probabilistic Intelligence Engine (PIE) (2026-02-11)

Advanced statistical and ML subsystems for agent behavior analysis: Bayesian Online Change-Point Detection (BOCPD), conformal prediction, cross-pane correlation with chi-squared co-occurrence, adaptive Kalman filter watchdog thresholds, ADWIN pattern drift detection, Bayesian evidence ledger, LSH error clustering, causal DAG with transfer entropy, session DNA behavioral fingerprinting, spectral FFT agent classification, MaxEnt IRL preference discovery, and VOI-optimal capture scheduling.

- [Bayesian Online Change-Point Detection](https://github.com/Dicklesworthstone/frankenterm/commit/df01ece958d6fe040f953fd3e0e78f8e5e834f4e)
- [Cross-pane correlation engine with chi-squared](https://github.com/Dicklesworthstone/frankenterm/commit/49a972c812e524013122d57f6fd547fabbc42115)
- [Causal DAG with transfer entropy](https://github.com/Dicklesworthstone/frankenterm/commit/260e9d2752e000fb87e609eebf4c25e816436128)
- [Session DNA behavioral fingerprinting with PCA](https://github.com/Dicklesworthstone/frankenterm/commit/ab30676b9f68cee704e5f41e1a00295163fa09df)
- [Spectral fingerprinting via FFT for agent classification](https://github.com/Dicklesworthstone/frankenterm/commit/51fd6060e2b7e313dd0a8f3c49005918baf88986)

### Search & Indexing Expansion (2026-02-19 -- 2026-02-21)

FrankenSearch subsystem: configurable fusion backend selector, embedding daemon server/worker, incremental document indexing pipeline, Tantivy-based lexical search service, daemon wire protocol, and WAL with CRC32 integrity and crash recovery.

- [Configurable fusion backend selector](https://github.com/Dicklesworthstone/frankenterm/commit/f86e760a04947c1c487a0a3b940db35c470e3e53)
- [Embedding daemon server and worker](https://github.com/Dicklesworthstone/frankenterm/commit/9e950a155cda968c833b98d422ae16d5923008d5)
- [Incremental document indexing pipeline](https://github.com/Dicklesworthstone/frankenterm/commit/1315fcea9960ec0f44dd02de54c450684ec67324)
- [WAL with CRC32 integrity and crash recovery](https://github.com/Dicklesworthstone/frankenterm/commit/12047c3c965e7050e84ffdf6344c66a5f40d96da)
- [TantivySearchService implementing LexicalSearchService trait](https://github.com/Dicklesworthstone/frankenterm/commit/46c0f9f053d45addd4fe2f1a32b8dddf3615bb63)

### Streaming Output & Mux Pool (2026-02-08 -- 2026-02-10)

Streaming output subscription from WezTerm mux, DirectMuxClient connection pool, mux watchdog integration, and backend-backed `get_text`/`list_panes`/`send_text`.

- [Streaming output subscription from WezTerm mux](https://github.com/Dicklesworthstone/frankenterm/commit/9b58b03be25f6025e3d5208823fcb4b265961346)
- [DirectMuxClient connection pool](https://github.com/Dicklesworthstone/frankenterm/commit/e7c06dde25917e3723bc5ed766a7a8a1ec3d5353)
- [Wire mux watchdog into watcher](https://github.com/Dicklesworthstone/frankenterm/commit/08b2d9fd4f3c6e6b30594af760e7995ac6a127f6)

### MCP Server Surface (2026-02-06 -- 2026-02-07)

MCP resource layer for agent introspection, URI-template resources, audit recording, and tool-level agent filtering.

- [MCP resource layer for agent introspection](https://github.com/Dicklesworthstone/frankenterm/commit/28ff3f745d440b2f0cb579f374ad04e55b454462)
- [URI-template resources and resource helpers](https://github.com/Dicklesworthstone/frankenterm/commit/a32f7819311ae186f9fa2ded0a8e25d607ecc28e)
- [MCP audit recording, query suggestions, and timeline](https://github.com/Dicklesworthstone/frankenterm/commit/776c0259e5246a2dcd79df3607229b45e7d4e934)

### Recording Engine & Secrets Scanner (2026-02-04)

Recording engine for session capture, secrets scanner with incremental checkpoint/resume, and Prometheus metrics endpoint.

- [Recording engine, secrets scanner, and incremental segment scan](https://github.com/Dicklesworthstone/frankenterm/commit/4f08087c3ce05aafd99ce329864a30d6c459c9bb)
- [Incremental scan with checkpoint/resume and schema v13 migration](https://github.com/Dicklesworthstone/frankenterm/commit/0dfcbded903dafc0c8735cd448a8703b92bd5e13)
- [Prometheus metrics endpoint support](https://github.com/Dicklesworthstone/frankenterm/commit/58e1ef1b2f81d658be2989d7c37cfb9df09610eb)

### IPC Authentication (2026-02-04)

Token-based authentication with scopes and expiry for IPC socket connections, plus RPC handler framework.

- [Token-based authentication with scopes and expiry](https://github.com/Dicklesworthstone/frankenterm/commit/773bca0672bdb7f649ac3ae083bf526262957ac1)
- [RPC handler framework and IPC client enhancements](https://github.com/Dicklesworthstone/frankenterm/commit/2a8bda4711696209ad0a23d2929d425d8d4b11ef)

### Data Structures Library (2026-02-12 -- 2026-02-22)

Comprehensive set of probabilistic and algorithmic data structures: bloom filter, ring buffer, reservoir sampler, token bucket rate limiter, exponential histogram, sharded counters, concurrent map, entropy accounting, homomorphic stream hashing, count-min sketch, bimap, sliding window, compact bitset, time series, edit distance, topological sort, shortest path, Fenwick tree, segment tree, union-find, and XOR filter.

- [Bloom filter](https://github.com/Dicklesworthstone/frankenterm/commit/cd0dbbe80fd3221feaa014245f6a893cc4050980)
- [Ring buffer for bounded circular storage](https://github.com/Dicklesworthstone/frankenterm/commit/a6bf85ae5cd37d48b4f684ae5c8842353ce9eddc)
- [Token bucket rate limiter](https://github.com/Dicklesworthstone/frankenterm/commit/edc2a0510a966ad67b47bf183b6d87ef60604817)
- [Sharded counters and concurrent map](https://github.com/Dicklesworthstone/frankenterm/commit/2f62916f9c4e80d686251bfa5b83b056734932f7)
- [Topological sort with graph algorithms](https://github.com/Dicklesworthstone/frankenterm/commit/4d3d9488040adb6483b0351ea58a7525d7364d2b)

### Latency Budget Framework (2026-02-23)

Latency stage decomposition, budget algebra, BudgetEnforcer, instrumentation probes with correlation context, adaptive budget allocator, three-lane scheduler with admission policy, bounded input ring with backpressure, priority inheritance, starvation prevention, zero-copy ingestion parser, and tail-latency controller.

- [Latency stage decomposition and budget algebra](https://github.com/Dicklesworthstone/frankenterm/commit/36752fb60baf799b85007ff1589c9e49a45fddf6)
- [Three-lane scheduler with admission policy](https://github.com/Dicklesworthstone/frankenterm/commit/524d24ee0f1ac71df8c1b6deeade7d1d10cd5114)
- [Zero-copy ingestion parser with line boundary detection](https://github.com/Dicklesworthstone/frankenterm/commit/9dc6e36361c604c462452d6bd73610698b2168d9)
- [Kernel/hardware tail-latency controller](https://github.com/Dicklesworthstone/frankenterm/commit/6431a258815ff6e56e42bee564d3301290b10ebf)

### Resize Subsystem (2026-02-13 -- 2026-02-14)

Resize scheduler with transaction state machine, cross-pane storm detection, domain-aware throttling, memory-pressure-aware controls, crash forensics with bundle integration, wrap quality scorecard, and resize dashboard.

- [Resize scheduler with transaction state machine](https://github.com/Dicklesworthstone/frankenterm/commit/2b42b9d8041467fc064a8a8a7b5d55d32f537454)
- [Cross-pane storm detection and domain-aware throttling](https://github.com/Dicklesworthstone/frankenterm/commit/8f7ce850a189c63ae5250c368db755cac6bf4743)
- [Resize crash forensics module](https://github.com/Dicklesworthstone/frankenterm/commit/259be6ba338a9602fa87b8f3af2462548e94b1ca)
- [Resize dashboard renderer with risk diagnostics](https://github.com/Dicklesworthstone/frankenterm/commit/f0486a913f049c44f5f47f0201b5b98687d0480f)

### Wire Protocol & Distributed Aggregator (2026-02-09)

Wire protocol aggregator for agent stream dedup and ingest, user pattern packs with custom namespace prefixes.

- [Aggregator for agent stream dedup and ingest](https://github.com/Dicklesworthstone/frankenterm/commit/204bf242031f0c6b2242250e7b2ff64a3922d982)
- [User pattern packs with custom namespace prefixes](https://github.com/Dicklesworthstone/frankenterm/commit/f63983088655528b0cbe6eb904db058383aff15d)

### Operational Telemetry (2026-02-10 -- 2026-02-14)

Per-pane process tree capture, FD budget tracking, memory pressure engine with tier-based actions, operational telemetry pipeline, Weibull survival model for mux health prediction, differential snapshot system, and p50/p95 rolling-window percentile telemetry.

- [Per-pane process tree capture](https://github.com/Dicklesworthstone/frankenterm/commit/90798df8758c714ee6eddfd66e5fdadc581817fe)
- [Memory pressure engine with tier-based actions](https://github.com/Dicklesworthstone/frankenterm/commit/98b6c349099bc518f3218f830e57c5ebdf56f340)
- [Weibull survival model for mux health prediction](https://github.com/Dicklesworthstone/frankenterm/commit/fab3dbaaac63f1440ab7a7d90c5c8bce19425cd8)
- [Differential snapshot system](https://github.com/Dicklesworthstone/frankenterm/commit/06742f973638a1b3f72546bbfa78dd2500ffa00a)

### Undo/Redo Framework (2026-02-08)

Undo/redo framework for reversible workflow actions with `wa undo` subcommand.

- [Undo/redo framework for reversible workflow actions](https://github.com/Dicklesworthstone/frankenterm/commit/fa91a9de78760582a865995807a5334d8b244c7f)
- [wa undo subcommand](https://github.com/Dicklesworthstone/frankenterm/commit/d77d5353b01c0447397350742a46efd06976d923)

### Comprehensive Proptest Coverage (2026-02-19 -- 2026-03-20)

Hundreds of property-based tests added across every major subsystem: search bridge (30), indexing (40), chunk vector store (30), WAL (CRC32/MuxMutation), format/table (35 each), bloom, ring buffer, count-min sketch, reservoir sampler, XOR filter, Fenwick tree, segment tree, union-find, and many more. Over 45,000 tests total.

- [30 property-based tests for search bridge](https://github.com/Dicklesworthstone/frankenterm/commit/b9f29177ff5cfe9273579ef8b298fc5aceb36f13)
- [40 property-based tests for search indexing](https://github.com/Dicklesworthstone/frankenterm/commit/390c95eb0257e858b8c38e758eabb904ca0ba3f0)
- [43-test proptest suite for swarm pipeline](https://github.com/Dicklesworthstone/frankenterm/commit/c8673900114f02d910d7480750e9e51105043768)
- [39 proptest serde roundtrips for uncovered cass types](https://github.com/Dicklesworthstone/frankenterm/commit/3455568402ddf46cc467edcf334527f8230b37c7)
- [24 behavioral runtime tests for core/vendored async boundary](https://github.com/Dicklesworthstone/frankenterm/commit/b604f80e03ad04ff1c7167a674fe413ea82639ae)

### Bug Fixes (selected)

- [Resolve ft status panic and ft watch lifecycle crash](https://github.com/Dicklesworthstone/frankenterm/commit/bbdbfdfbdc98801b00d8a826849bea92f171e148)
- [Harden byte compression, runtime abort, session resume, tx execution](https://github.com/Dicklesworthstone/frankenterm/commit/d961feaae53abcf631cc002422fbe1f7bb40912c)
- [Rate limit off-by-one, silent compression failure, silenced ledger errors](https://github.com/Dicklesworthstone/frankenterm/commit/c948158e44cbe074dc4018817568a390927ea0c1)
- [Clamp WebGPU surface dimensions to prevent zero-size panics](https://github.com/Dicklesworthstone/frankenterm/commit/a9e5b076f6f75b35349551c60f00442934c88ee6)
- [Validate PaneDelta content_len matches actual content length](https://github.com/Dicklesworthstone/frankenterm/commit/79731b1b5ab62a4860f3734d8f7cbecdc78182a4)
- [Harden wire protocol, storage, and session restore with strict validation](https://github.com/Dicklesworthstone/frankenterm/commit/bdd966091dc864be1885b1723c10fc20ef8528b4)
- [Use constant-time comparison for identity validation](https://github.com/Dicklesworthstone/frankenterm/commit/9d2d038f9156b9676dc0b3b8d64fe21f475a88ba)
- [CxScope RAII guard for context lifecycle](https://github.com/Dicklesworthstone/frankenterm/commit/2f5ed9bf85680bc4236049234ec9891a007b2a17)
- [RAII MuxSubscriptionGuard to prevent subscription leaks](https://github.com/Dicklesworthstone/frankenterm/commit/ffc814429896fcd100c8b8bdf4ed3c913af0bdc3)

### BREAKING: Remove Lua-based status update hook (2026-01-28)

Removed `update-status` Lua callback that fired at ~60Hz, causing continuous overhead. Alt-screen detection now via escape sequence parsing. Pane metadata via polling only when needed.

- [Remove Lua status update hook for performance](https://github.com/Dicklesworthstone/frankenterm/commit/ff24bb3f23be958f5fec88725ceb8aebb819d9aa)
- [Remove StatusUpdateReceived event variant](https://github.com/Dicklesworthstone/frankenterm/commit/ee66abdf20bfb90d8e0d99117fdcf5b19f8b66ac)

---

## [0.1.0] — 2026-01-25

> Initial release. 469 commits across 2026-01-18 to 2026-01-25. Originally named `wezterm_automata` (`wa`), later renamed to `frankenterm` (`ft`).

### Core Platform

- **Rust workspace** with strict safety settings (`forbid(unsafe_code)`) and comprehensive lint configuration
  - [Configure workspace with strict safety and lints](https://github.com/Dicklesworthstone/frankenterm/commit/2df6aa108990b67fe9c775fc2a6ff12fb5549437)
- **WezTerm client and domain models** for pane discovery, fingerprinting, and lifecycle tracking
  - [Core library with WezTerm client and domain models](https://github.com/Dicklesworthstone/frankenterm/commit/0b3d4a9db84de4660caae6c0464093d5720adbbc)
- **CLI binary** with command structure and Robot Mode
  - [CLI binary with command structure and robot mode](https://github.com/Dicklesworthstone/frankenterm/commit/fffc22f85d5a1910805e52bc18df6e559822bf0c)

### Pattern Detection Engine

- Multi-agent pattern engine detecting rate limits, errors, prompts, and completions across Codex, Claude Code, and Gemini
- DetectionContext for agent filtering and deduplication
- Golden corpus regression harness for pattern validation
  - [DetectionContext for agent filtering and deduplication](https://github.com/Dicklesworthstone/frankenterm/commit/d40e9a22d7bdf6b25171a2ec15361ecdeee44e4a)
  - [Golden corpus regression harness](https://github.com/Dicklesworthstone/frankenterm/commit/ea97e8f3120eac4e34f5da1d967c57be9ae513a5)

### Robot Mode API

- JSON/TOON interface optimized for AI agent orchestration
- Consistent response schema: `ok`, `data`, `error`, `elapsed_ms`, `version`
- TOON output format for 40-60% token savings in AI-to-AI communication
- Robot commands: `state`, `get-text`, `wait-for`, `search`, `send`, `events`
  - [Extensive robot mode enhancements](https://github.com/Dicklesworthstone/frankenterm/commit/9a3066ed0f603e40eb75f82dd5e4a3674506bae0)
  - [Robot JSON schemas](https://github.com/Dicklesworthstone/frankenterm/commit/a7e91cdc177995a9562ce0844e0c919e13964160)

### Full-Text Search

- FTS5-backed search across all captured pane output with BM25 ranking and snippets
  - [FTS search API with BM25 ranking and snippets](https://github.com/Dicklesworthstone/frankenterm/commit/6cd591f978080995d0a0a4f32d838c16c14d2f71)

### Storage

- Comprehensive SQLite schema for events, patterns, panes, and captures
  - [Comprehensive SQLite schema](https://github.com/Dicklesworthstone/frankenterm/commit/7ea47c64adea16a63f4b8ef9da2a86354f46fdb1)

### Policy Engine

- ActionKind, PolicyDecision, authorize() API with capability gates and rate limiting
- PolicyGatedInjector for unified input injection with audit trail
  - [Policy model with authorize() API](https://github.com/Dicklesworthstone/frankenterm/commit/6f0ba75cf5de85d04cf6d1211346c7e2f1ee756a)
  - [PolicyGatedInjector for unified input injection](https://github.com/Dicklesworthstone/frankenterm/commit/e83554947dda13615bed63577a7bad33c85c0318)

### Delta Extraction

- Capture snapshot method and delta extraction with 4KB overlap matching
- Gap recording for sequence discontinuity detection
  - [Capture snapshot and delta extraction](https://github.com/Dicklesworthstone/frankenterm/commit/1a668cde22cebf7fd640d560a6018e8aab95f065)

### Ingestion & Observation

- Pane discovery with fingerprinting and lifecycle tracking
- OSC 133 semantic prompt marker parsing
- ObservationRuntime for passive pane monitoring
- Event bus with bounded channels and fanout
  - [Pane discovery with fingerprinting](https://github.com/Dicklesworthstone/frankenterm/commit/03d5c101970f7ab4a52839b40e68efd973f0f1f1)
  - [ObservationRuntime for passive monitoring](https://github.com/Dicklesworthstone/frankenterm/commit/68c24fff31b9c34236b55e239479bf8f3b0b69bc)
  - [Event bus with bounded channels and fanout](https://github.com/Dicklesworthstone/frankenterm/commit/a1ca9a8a80986f35527f75f152ca6786a070bd11)

### Workflow Engine

- Workflow trait, WorkflowContext, per-pane workflow locks, and scheduling
  - [Workflow trait and types](https://github.com/Dicklesworthstone/frankenterm/commit/f7708aa4cbf318ec2282d6963a41bad61518d7e2)
  - [Comprehensive workflow execution with runner](https://github.com/Dicklesworthstone/frankenterm/commit/a3b7244cd4588208b68f000d68b89e55614db717)

### IPC & Runtime

- Unix socket IPC for watcher daemon communication
- TailerSupervisor for adaptive pane polling
- Crash recovery module
- Hot-reload config broadcasting
  - [Unix socket IPC](https://github.com/Dicklesworthstone/frankenterm/commit/da3ba590bf48ef4d75b384caf818de6dbd551e61)
  - [TailerSupervisor for adaptive pane polling](https://github.com/Dicklesworthstone/frankenterm/commit/dc7aa9bdac59ba31858b165c869b174c71fa1ccb)
  - [Crash recovery module](https://github.com/Dicklesworthstone/frankenterm/commit/0cdef4d265e25ab65043bb2fc606a38ea768d4a6)
  - [Hot-reload config broadcasting](https://github.com/Dicklesworthstone/frankenterm/commit/f2b086e3aa6ba53407e0f472e9c347c6a75fc75b)

### CI/CD & Testing

- GitHub Actions CI/CD workflows
- Criterion benchmarks for critical paths
- Daemon integration tests with synthetic deltas
- E2E test harness
  - [GitHub Actions CI/CD workflows](https://github.com/Dicklesworthstone/frankenterm/commit/31110d4277e13408d2eb099ff2d3372cbe725382)
  - [Criterion benchmarks](https://github.com/Dicklesworthstone/frankenterm/commit/ab5d821775acaed038897f7a326efda572749ef6)
  - [E2E test harness](https://github.com/Dicklesworthstone/frankenterm/commit/d4cd86177bc0bb7cf8ed1f3683f2f14053c36139)

---

## Tags & Releases

| Tag / Ref | Type | Date | Points to | Description |
|-----------|------|------|-----------|-------------|
| `backup-before-rewrite` | Git tag (no GitHub Release) | 2026-02-17 | [`888c17d0`](https://github.com/Dicklesworthstone/frankenterm/commit/888c17d0da2564269df114e4c5d9ecfd8edf85c5) | Snapshot before the major WezTerm source import and codebase rewrite |

There are no GitHub Releases published for this repository.

---

## Project Timeline

| Date | Milestone |
|------|-----------|
| 2026-01-18 | First commit. Workspace setup, core library, CLI binary, WezTerm client. |
| 2026-01-19 | Rapid feature buildout: FTS, events, patterns, policy, workflows, robot mode, IPC, CI. |
| 2026-01-25 | v0.1.0 feature set complete. Agent friendliness report, suggestions engine. |
| 2026-01-27 | curl-bash installer, doctor command, action plans, risk scoring. |
| 2026-01-28 | Remove Lua status hook (BREAKING). SSH setup, chaos testing, backup export/import. |
| 2026-01-29 | Triage command, pane reservations, browser automation, CASS CLI, data export. |
| 2026-02-04 | Recording engine, secrets scanner, IPC auth, Prometheus metrics, MCP. |
| 2026-02-08 | FTUI migration complete. Undo/redo framework. Streaming mux subscription. |
| 2026-02-09 | Wire protocol aggregator. Distributed mode readiness. DirectMuxClient pool. |
| 2026-02-10 | **WezTerm source import.** Rename wa -> ft. Session persistence. |
| 2026-02-11 | PIE (Probabilistic Intelligence Engine): BOCPD, causal DAG, session DNA. |
| 2026-02-12 | Data structures library. Flight recorder. Runtime compat layer. |
| 2026-02-13 | WASM extension system. Resize subsystem. Config migration tool. |
| 2026-02-17 | `backup-before-rewrite` tag. Asupersync runtime abstraction begins. |
| 2026-02-19 | FrankenSearch: fusion backend, embedding daemon, WAL, 100+ proptests. |
| 2026-02-22 | Tool integration bridges (beads_rust, UBS, vibe_cockpit). Recorder backend-agnostic seam. |
| 2026-02-23 | Latency budget framework. ARS (Autonomous Reflex System). |
| 2026-03-01 | Dashboard aggregator. Cost tracker with budget alerts. |
| 2026-03-02 | **Native GUI terminal.** Mux server. FrankenTerm.app builds from source. |
| 2026-03-03 | Swarm orchestration runtime. Connector SDK. Native mux lifecycle. |
| 2026-03-10 | 21-subsystem policy engine. Forensic export pipeline. |
| 2026-03-12 | Tiered scrollback. Fleet memory controller. 200-pane stress tests. |
| 2026-03-13 | Transaction execution engine. Input-to-display latency framework. |
| 2026-03-17 | Distributed checkpoint save/restore. Replay forensics with sensitivity tiers. |
| 2026-03-20 | CASS export feature. 92+ proptest serde roundtrip suites. |
| 2026-03-21 | HEAD. 3,928 commits. ~775k lines of code. 120 workspace crates. 45,000+ tests. |

---

<!-- Links -->
[Unreleased]: https://github.com/Dicklesworthstone/frankenterm/compare/backup-before-rewrite...main
[0.1.0]: https://github.com/Dicklesworthstone/frankenterm/commits/main/?after=backup-before-rewrite
