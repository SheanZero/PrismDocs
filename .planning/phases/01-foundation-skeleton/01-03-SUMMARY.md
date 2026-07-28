---
phase: 01-foundation-skeleton
plan: 03
subsystem: database
tags: [sqlite, fts5, trigram, wal, rusqlite, rusqlite_migration, r2d2, query_only, schema-v1]

# Dependency graph
requires:
  - "01-01：prism-store 的 tracer 版 Store::open / data_root / default_db_path，以及根 Cargo.toml 的版本 pin"
  - "01-02：scripts/check-deps.sh 的四条依赖方向断言（本 plan 新增依赖后必须仍然全绿）"
provides:
  - "migration 001（schema v1）：projects / documents / settings 三张 STRICT 表 + documents_fts external-content FTS5 虚拟表 + documents_ai/ad/au 三个同步触发器"
  - "prism_store::migrations()：LazyLock<Migrations> 迁移集合，后续 phase 只 append"
  - "prism_store::open() / Store：writer-first 六步启动序列，单写者 Mutex<Connection> + query_only 只读池"
  - "Store::write / Store::read 闭包式 API（连接不可长持有，写锁不可跨 await）"
  - "Store::close()：先弃池再做 TRUNCATE checkpoint"
  - "常量 BUSY_TIMEOUT_MS / MIN_SQLITE / READ_POOL_MAX_SIZE"
  - "StoreError 独立模块，含 Pool / Migration / SqliteTooOld 变体，Display 不含路径"
affects: [01-04, 01-05, 01-08, phase-2-导入与搜索, phase-3-锚定, phase-5-评论, phase-7-卡片]

# Tech tracking
tech-stack:
  added:
    - "r2d2 0.8 + r2d2_sqlite 0.35（只读连接池，with_init 注入每连接 pragma）"
    - "rusqlite_migration 2.6（user_version 追踪的迁移体系）"
    - "ulid 1 / blake3 1.8 / serde 1 / url 2（schema v1 的列语义所需，Phase 2 起消费）"
  patterns:
    - "writer-first 六步序：writer 先行建文件 → WAL → 每连接 pragma → 版本校验 + 迁移 → 建只读池 → close 时 TRUNCATE checkpoint"
    - "external-content FTS + 三触发器：索引同步交给数据库，写路径无从遗漏"
    - "闭包式 store API：write(|tx|) / read(|conn|)，调用方拿不到可长持有的句柄"
    - "错误类型不携带路径与用户内容（T-01-20）"

key-files:
  created:
    - crates/prism-store/migrations/001_schema_v1.sql
    - crates/prism-store/src/migrations.rs
    - crates/prism-store/src/open.rs
    - crates/prism-store/src/error.rs
    - crates/prism-store/tests/concurrency.rs
    - .planning/phases/01-foundation-skeleton/deferred-items.md
  modified:
    - crates/prism-store/Cargo.toml
    - crates/prism-store/src/lib.rs

key-decisions:
  - "schema v1 采用方案 A（用户 checkpoint 决策）：external-content FTS5 + documents.rowid_pk 显式 INTEGER PRIMARY KEY + 三同步触发器 + STRICT 表 + trigram + 索引粒度保持默认全粒度"
  - "索引粒度由省略做出选择：001_schema_v1.sql 刻意不声明该选项。降粒度会禁掉长度超过 3 个 unicode 字符的全文查询，D-01 的 4 字中文词当场失效——而 external content 省下的体积让「为省体积而降粒度」的动机本就不存在"
  - "只读池用 SQLITE_OPEN_READ_WRITE + query_only=ON，而不是 SQLITE_OPEN_READ_ONLY：只读 flags 的连接在崩溃后 -shm 缺失时无法重建它，会拿到 SQLITE_CANTOPEN"
  - "close() 先 drop 只读池再 checkpoint：TRUNCATE checkpoint 在还有活连接时会「成功但没做事」（返回 busy 标志而非报错），这正是该类 bug 静默的地方"
  - "StoreError::Io 不再携带 PathBuf（原 tracer 版把绝对路径写进了 Display）——T-01-20 的直接落地"
  - "Store::open 保留为自由函数 open() 的委托，避免改动 prism-engine 的既有调用点"
  - "计划设想的第二条反证（颠倒迁移与建池顺序使 reader_sees_migrated_schema 变红）**实测不成立**；改用源码顺序断言作为常驻哨兵（见 Issues Encountered）"

patterns-established:
  - "已发布的迁移不可修改：后续 phase 只在 MIGRATIONS 的 vec! 末尾 append"
  - "迁移文件内不出现连接级设置语句——那属于打开流程，混进迁移会让 validate() 的内存库行为与真实库分叉"
  - "每条不可逆约束都配一条反证：删掉它必须让对应测试变红，否则断言可能恒真"
  - "行为测不到的约束（如源码步骤顺序）用结构断言兜底，而不是留在注释里"

requirements-completed: [INFRA-02]

coverage:
  - id: D1
    description: "迁移体系可用：migrations() 的 validate() 通过，to_latest 幂等"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/migrations.rs#migrations_are_valid"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/migrations.rs#to_latest_is_idempotent"
        status: pass
    human_judgment: false
  - id: D2
    description: "schema v1 落地：四个对象（projects/documents/documents_fts/settings）+ 三个触发器 + rowid_pk 为显式 INTEGER PRIMARY KEY"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/migrations.rs#schema_v1_creates_the_four_minimal_objects"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/migrations.rs#schema_v1_creates_the_three_fts_sync_triggers"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/migrations.rs#documents_rowid_pk_is_an_explicit_integer_primary_key"
        status: pass
    human_judgment: false
  - id: D3
    description: "FTS 索引与内容表在 INSERT / UPDATE / DELETE 三条路径上均由触发器保持同步（方案 A 静默失败模式的兜底）"
    requirement: "INFRA-02"
    verification:
      - kind: unit
        ref: "crates/prism-store/src/migrations.rs#fts_index_stays_in_sync_across_insert_update_and_delete"
        status: pass
      - kind: other
        ref: "反证：注释掉 documents_ad 触发器后该测试在 delete 断言处变红；恢复后转绿"
        status: pass
    human_judgment: false
  - id: D4
    description: "WAL 下读者快照与写者提交互不阻塞，读者全程无 SQLITE_BUSY"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#reader_snapshot_is_isolated"
        status: pass
    human_judgment: false
  - id: D5
    description: "只读池的每一条连接都写不进去（query_only=ON 由 with_init 逐连接注入）"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#pooled_connection_cannot_write"
        status: pass
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#every_pooled_connection_is_query_only"
        status: pass
      - kind: other
        ref: "反证：删除 with_init 中的 query_only=ON 后上述两个测试变红（4 passed / 2 failed），恢复后 6 全绿"
        status: pass
    human_judgment: false
  - id: D6
    description: "bundled SQLite ≥ 3.51.3，且 close() 后 -wal 被截断"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#bundled_sqlite_meets_minimum"
        status: pass
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#wal_truncated_on_close"
        status: pass
    human_judgment: false
  - id: D7
    description: "迁移先于只读池建立：池连接看到的是已迁移 schema，且源码顺序有常驻哨兵"
    requirement: "INFRA-02"
    verification:
      - kind: integration
        ref: "crates/prism-store/tests/concurrency.rs#reader_sees_migrated_schema"
        status: pass
      - kind: unit
        ref: "crates/prism-store/src/open.rs#migration_runs_before_the_read_pool_is_built"
        status: pass
    human_judgment: true
    rationale: "行为面只证明了「迁移跑过」，没证明「跑在建池之前」——计划设想的顺序反证实测不成立（见 Issues Encountered）。结构断言是替代哨兵，其充分性需人工确认。"

# Metrics
duration: 68min
completed: 2026-07-29
status: complete
---

# Phase 01 Plan 03: 连接架构与 schema v1 Summary

**writer-first 六步启动序列 + 单写者 `Mutex<Connection>` / query_only 只读池，配 migration 001 的 external-content trigram FTS5（三触发器同步、全粒度索引、STRICT 表）**

## Performance

- **Duration:** ~68 min
- **Started:** 2026-07-28T22:03:00Z
- **Completed:** 2026-07-28T23:11:00Z
- **Tasks:** 2（另有 1 个 checkpoint 决策由用户在上一轮裁定）
- **Files created/modified:** 8

## Accomplishments

- **migration 001 落地且不可再改**：`projects` / `documents` / `settings` 三张 STRICT 表 +
  `documents_fts`（`content='documents'`、`content_rowid='rowid_pk'`、`tokenize='trigram'`）+
  `documents_ai` / `documents_ad` / `documents_au` 三个同步触发器。索引粒度保持 SQLite 默认的全粒度
  ——文件里刻意不出现该选项，因为降粒度会让 4 字中文词的 MATCH 直接失效。
- **索引同步责任搬进了数据库**：这是方案 A 的全部意义。Phase 2/3/5/7 新增的任何写路径都不必
  「记得同步 FTS」，因为它们没有机会忘记。
- **writer-first 六步序**：writer 先行建文件 → WAL（持久设置，只设一次）→ 每连接 pragma 套餐 →
  版本下限校验 + 裸 Connection 上跑迁移 → 只读池（`with_init` 末位注入 `query_only=ON`）→
  `close()` 弃池后 TRUNCATE checkpoint。
- **并发纪律有反证支撑**：删掉 `query_only=ON` 会让两个池测试变红；注释掉 `documents_ad`
  会让触发器同步测试变红。两条断言都不是恒真的。
- **`prism-store` 测试从 4 涨到 17**（11 lib + 6 concurrency），workspace 从 33 涨到 46，全绿。

## Task Commits

1. **Task 1: 迁移体系与 schema v1** — TDD 三段
   - `65d75b5` (test) — 空迁移集 + 五个失败测试（RED：5 failed）
   - `875b9c8` (feat) — `001_schema_v1.sql` + `include_str!` 接线（GREEN：5 passed）
   - `43a9a0b` (test) — INSERT/UPDATE/DELETE 三路径触发器同步测试（Rule 2 补充，见 Deviations）
2. **Task 2: writer-first 六步序与并发纪律** — TDD 两段
   - `da41674` (test) — `tests/concurrency.rs` 六个测试写在 API 存在之前（RED：编译失败，且失败被隔离在该测试目标内，`prism-engine` 未受影响）
   - `ddbde85` (feat) — `src/open.rs` 实现 + `lib.rs` 改为模块聚合器（GREEN：11 + 6 全绿）

## Files Created/Modified

- `crates/prism-store/migrations/001_schema_v1.sql` — schema v1。**已发布，不可修改**
- `crates/prism-store/src/migrations.rs` — `LazyLock<Migrations>` + `migrations()` + 六个单测
- `crates/prism-store/src/open.rs` — 六步启动序列、`Store` 的 write/read/close/sqlite_version、三个常量、源码顺序哨兵测试
- `crates/prism-store/src/error.rs` — `StoreError` 独立模块（Io / Sqlite / Pool / Migration / SqliteTooOld / NoDataDir）
- `crates/prism-store/tests/concurrency.rs` — 六个并发集成测试，各用独立 `tempfile::tempdir()`
- `crates/prism-store/src/lib.rs` — 改为模块聚合器，保留 `data_root` / `default_db_path`
- `crates/prism-store/Cargo.toml` — 补齐 r2d2 / r2d2_sqlite / rusqlite_migration / ulid / blake3 / serde / url
- `.planning/phases/01-foundation-skeleton/deferred-items.md` — 范围外发现登记

## Decisions Made

除上文 frontmatter 的 `key-decisions` 外，两处值得单独说明：

- **`Store::open` 保留为 `open()` 的委托。** 计划的 `lib.rs` 形态是 `pub use open::{Store, open}`，
  但 `prism-engine` 的三个既有测试调的是 `Store::open`。加一层单行委托比改兄弟 crate 的调用点
  代价小，也让两种写法都成立。
- **`documents_fts` 不带 `STRICT`。** 计划正文写的是「四张表均带 STRICT」，但 `documents_fts`
  是虚拟表，SQLite 不接受虚拟表的 `STRICT` 修饰。实际形态是三张真实表 STRICT + 一张虚拟表——
  即计划所说的「四个对象」。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] 补上 INSERT/UPDATE/DELETE 三路径的触发器同步测试**
- **Found during:** Task 1（迁移体系与 schema v1）
- **Issue:** 方案 A 自身登记的唯一弱点是「触发器出错时表现为搜不到而非报错」。计划把这项兜底
  指给了 plan 01-05 的 `fts_cjk` 用例，但 01-05 尚未执行，且计划 Task 1 的 `<behavior>` 只断言
  触发器**存在**（数 `sqlite_master`），不断言它们**做对了事**。三个触发器全部写成空体也能通过。
- **Fix:** 在 `src/migrations.rs` 加 `fts_index_stays_in_sync_across_insert_update_and_delete`，
  逐条走 INSERT → UPDATE（delete 半边 + insert 半边）→ DELETE，并用 4 字中文词
  「锚定引擎」同时验证索引粒度未被降级。
- **Files modified:** `crates/prism-store/src/migrations.rs`
- **Verification:** 注释掉 `documents_ad` 触发器后测试在 delete 断言处变红；恢复后转绿。
- **Committed in:** `43a9a0b`

**2. [Rule 2 - Missing Critical] `StoreError::Io` 移除绝对路径**
- **Found during:** Task 1（error.rs 拆分）
- **Issue:** plan 01-01 的 tracer 版把 `path: PathBuf` 写进了 `Io` 变体的 `Display`
  （`#[error("io error at {path}: {source}")]`）。威胁 T-01-20 明确要求错误不得携带数据库绝对路径。
- **Fix:** 改为 `Io(#[from] std::io::Error)`；`std::io::Error` 的 `Display` 只给 errno 描述，不含路径。
- **Files modified:** `crates/prism-store/src/error.rs`、`crates/prism-store/src/lib.rs`
- **Verification:** `cargo test -p prism-store` 全绿；变体已无 `PathBuf` 字段。
- **Committed in:** `65d75b5`

**3. [Rule 2 - Missing Critical] 为「迁移先于建池」补一条常驻结构断言**
- **Found during:** Task 2（执行计划要求的第二条反证时）
- **Issue:** 计划的验证段要求「颠倒迁移与建池顺序使对应测试变红」。**实测不成立**——把建池挪到
  WAL 与迁移之前，`tests/concurrency.rs` 六个测试依然全绿。原因是两者指向同一个库文件路径，
  池连接会在迁移提交后重读 schema cookie，照样看得见新表。也就是说
  `reader_sees_migrated_schema` 守的是「迁移到底跑没跑」（去掉 `to_latest` 它立刻变红，
  连带另外四个也红），守不住「跑在建池之前」。这一步于是成了六步序里唯一没有哨兵的一步。
- **Fix:** 在 `src/open.rs` 加 `migration_runs_before_the_read_pool_is_built`，用 `include_str!`
  读自身源码，断言 `to_latest(&mut writer)` 的位置早于 `Pool::builder()`。计划原本把这条
  留作一次性的 grep 验收项；改成测试后它在 CI 里常驻。
- **Files modified:** `crates/prism-store/src/open.rs`
- **Verification:** 测试通过；把两处调换即变红。
- **Committed in:** `ddbde85`

### 其他偏离

- **`tokio::sync::Mutex` 出现 0 次**这条验收项原本被一句解释性文档注释命中（注释内容正是
  「用 `std::sync::Mutex` 而不是它」）。改写为「tokio 的异步 Mutex」，语义不变，机械检查通过。

---

**Total deviations:** 3 auto-fixed（全部 Rule 2 - 缺失的关键性）+ 1 处措辞调整
**Impact on plan:** 三条都是补哨兵而非改设计，没有引入计划外的功能面。方案 A 的形态与计划完全一致。

## Issues Encountered

- **计划设想的顺序反证不成立**（详见上方 Deviations #3）。这是本次执行最值得记下的一条：
  「颠倒顺序应当让某测试变红」是计划阶段的推断，实跑不支持它。没有跑这条反证的话，
  会带着一条自以为被守住、实际没有的约束进入 Phase 2。**反证本身也需要被验证。**
- `cargo fmt --all -- --check` 在 workspace 多处报差异，但 `.github/workflows/ci.yml` 没有 fmt 步骤，
  差异也早于本 plan。按 scope boundary 未修，已登记到
  `.planning/phases/01-foundation-skeleton/deferred-items.md`。

## Known Stubs

None——本 plan 无占位实现。`ulid` / `blake3` / `serde` / `url` 四个依赖按计划提前进入
`prism-store` 的依赖面（Phase 2 起消费），但没有对应的空实现或假数据。

## Threat Flags

None——本 plan 未引入计划 `<threat_model>` 之外的安全面。三条 `mitigate` 处置
（T-01-02 / T-01-19 / T-01-20）均已落地并各有测试或反证；T-01-09 由单写者 + 版本下限 +
plan 01-02 的 dup 断言共同覆盖；T-01-10 按计划 `accept`，量化复核留到 Phase 8。

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- **plan 01-04 / 01-05 可以开始**：`Store::write` / `Store::read` 与 schema v1 已就位，
  01-05 的 `fts_cjk` 判别性用例（4 字中文 MATCH、VACUUM 后仍命中、阴性对照）建在其上。
  注意 01-05 的 VACUUM 用例是 `rowid_pk` 显式主键设计的直接验收，务必保留。
- **plan 01-08（命令层）** 消费 `StoreError` 时，注意它已不再携带路径——命令层的二次映射
  只需处理类别，不必再做脱敏。
- **写给 Phase 2 的一句话**：schema v1 已冻结。新表一律走新的迁移文件（`002_*.sql`），
  `001_schema_v1.sql` 不得再改一个字符。
- **遗留判断题**：`reader_sees_migrated_schema` 的守备范围比计划设想的窄（见 Issues）。
  若 `/gsd-verify-work` 认为源码顺序断言不足以替代行为哨兵，需要另想办法——但目前没有
  已知的行为面手段能区分这两种顺序。

---
*Phase: 01-foundation-skeleton*
*Completed: 2026-07-29*

## Self-Check: PASSED

- 9 个声明的文件全部在盘上（含 SUMMARY 与 deferred-items）
- 6 个声明的提交全部在 git 历史中（65d75b5 / 875b9c8 / 43a9a0b / da41674 / ddbde85 / 7753fbc）
- `cargo test -p prism-store` 11 + 6 全绿；`cargo test --workspace` 46 passed / 0 failed
- `bash scripts/check-deps.sh` 与 `bash scripts/check-secrets.sh` 均 exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` 无输出（exit 0）
