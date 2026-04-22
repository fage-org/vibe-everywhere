# Session Rerun/Resume Review Open Issues

**Reviewed**: 2026-04-20  
**Scope**: `session rerun/resume` ECC 复核中已发现但尚未修复的问题  
**Note**: 本文只记录未关闭项；已修复项不重复展开。

## 按模块记录

### 1. `ve-server` API / Session control

#### HIGH-01: archived rerun 在本地入队后就变成可复用 `pending`

- **功能链路**: archived session -> rerun -> server dispatch -> daemon accept
- **位置**: `crates/ve-server/src/api/sessions.rs:891`, `crates/ve-server/src/api/sessions.rs:926`, `crates/ve-server/src/api/sessions.rs:954`
- **现状**:
  - `handle_archived_rerun()` 先把新 rerun session 插入为 `dispatching`
  - 随后只要 `send_to_daemon()` 返回 true，就在 `crates/ve-server/src/api/sessions.rs:926` 把状态改成 `pending`
  - 但这里的 true 只代表消息进入本地 channel，不代表 daemon 已实际接收、启动或确认该 rerun
- **风险**:
  - 后续相同 archived session 的 rerun 请求会在 `find_reusable_rerun_session_id()` 中复用这个 session
  - 如果 daemon 侧实际上没有成功接单，server 会把一个并未真正 ready 的 session 暴露成可复用 live session
  - 这会破坏 rerun 幂等语义，也会让调用方看到错误的“已有可复用 rerun”结论
- **为什么仍未关闭**:
  - 当前还没有把 `dispatching -> pending` 的时机绑定到 daemon 正向确认事件
  - 该问题已经在 review 中被判为 HIGH，修复方案需要继续按 ECC 决策推进

#### HIGH-02: `create_session` 严格幂等仍存在并发竞态

- **功能链路**: create session -> idempotency lookup -> session insert -> idempotency store
- **位置**: `crates/ve-server/src/api/sessions.rs:232`, `crates/ve-server/src/api/sessions.rs:323`, `crates/ve-server/src/api/sessions.rs:351`, `crates/ve-server/src/db/idempotency.rs:124`
- **现状**:
  - `create_session()` 先 `store.get()` 查重，再直接插入 `sessions`
  - 插入 session 和写入 `idempotency_keys` 不在同一个原子事务里
  - `IdempotencyKeyStore::store()` 在主键冲突时会返回既有 key 对应结果，但不会回滚已经新建的额外 session
- **风险**:
  - 两个并发相同 idempotency key 的请求可能同时 miss `get()`
  - 之后可能创建出两个 session，只有一个被 idempotency key 记录引用，另一个变成“幽灵 session”
  - 这与“strict idempotency”目标不一致
- **为什么仍未关闭**:
  - 当前实现仍是“先查再写、分步落库”的结构
  - 本轮只处理了 rerun/resume 主链路，尚未把 create_session 也收敛到原子幂等语义

#### MEDIUM-01: rerun 唯一性路径缺少真实并发 / PostgreSQL 级验证

- **功能链路**: archived session -> rerun idempotency -> unique index / conflict fallback
- **位置**: `crates/ve-server/src/api/sessions.rs:816`, `crates/ve-server/src/api/sessions.rs:852`
- **现状**:
  - 当前代码已依赖唯一索引冲突和 fallback 查询来维持 rerun 幂等
  - 但本轮可见验证仍主要集中在单机 SQLite 路径与定向逻辑测试
  - 尚未补到能稳定覆盖“并发 rerun 冲突 + PostgreSQL 行为”的回归验证
- **风险**:
  - 这类逻辑对数据库错误文本、唯一索引时机、并发窗口都较敏感
  - 若只在 SQLite 单路径验证通过，仍可能遗漏 PostgreSQL 或高并发下的行为偏差
- **为什么仍未关闭**:
  - 当前属于已知验证缺口，尚未看到对应的并发 / Postgres 回归用例补齐

### 2. `ve-server` SQLite migrations

#### HIGH-03: SQLite migration 006 重建 `sessions` 时会丢失 `rerun_from_session_id`

- **功能链路**: SQLite startup migration replay -> sessions rebuild -> rerun provenance preservation
- **位置**: `crates/ve-server/src/db/migrations/sqlite/006_session_pending_status.sql:21`
- **现状**:
  - migration 006 通过 `sessions_new` 重建表结构
  - 新表定义中没有 `rerun_from_session_id`
  - `INSERT INTO sessions_new ... SELECT ... FROM sessions` 也没有搬运该字段
- **风险**:
  - 在当前工程“启动时重跑 migration”的语义下，已有 rerun provenance 可能被重置成 `NULL`
  - 这会直接破坏 archived rerun 的来源追踪和后续幂等判断
- **为什么仍未关闭**:
  - 本轮新增的 rerun 字段是在 migration 007 引入的，但 migration 006 的重建脚本未同步升级
  - 该问题已经在 review 中被判为 HIGH，尚未修复

#### MEDIUM-02: SQLite migration 002 仍是逐语句、非原子补丁式执行

- **功能链路**: SQLite startup migration replay -> supplemental fields patching
- **位置**: `crates/ve-server/src/db/mod.rs:234`, `crates/ve-server/src/db/mod.rs:262`
- **现状**:
  - `run_sqlite_migration_002()` 用 `STEPS` 数组逐条执行 SQL
  - 对部分错误按字符串匹配直接视为“already applied”并跳过
  - 整个 migration 没有单次原子重建或显式 schema version 收敛点
- **风险**:
  - 一旦某次启动中途失败，数据库可能停留在“部分步骤已执行”的中间态
  - 下次启动再靠字符串匹配补跑，容易出现状态漂移，且很难证明所有目标结构都已正确收敛
- **为什么仍未关闭**:
  - 该 migration runner 设计仍保留补丁式重放行为
  - 本轮尚未把 migration 002 收敛成更强的一致性方案

## 按功能链路记录

### A. Archived session rerun / resume 链路

#### HIGH-A1: server 把“已入本地 channel”误判成“daemon 已接受 rerun”

- **模块**: `ve-server` API
- **位置**: `crates/ve-server/src/api/sessions.rs:894`, `crates/ve-server/src/api/sessions.rs:926`
- **问题摘要**:
  - rerun session 只要成功进入本地发送通道，就会被改成 `pending`
  - 这早于 daemon 正向 ack / ready 语义，导致该 session 可被后续请求过早复用

#### MEDIUM-A2: rerun 幂等的数据库竞争路径仍缺少并发 / PostgreSQL 回归证明

- **模块**: `ve-server` API + DB constraints
- **位置**: `crates/ve-server/src/api/sessions.rs:816`, `crates/ve-server/src/api/sessions.rs:852`
- **问题摘要**:
  - 当前实现依赖唯一索引 + 冲突回退，但验证层面对真实并发和 PostgreSQL 仍不足
  - 该项尚未形成足够强的回归保证

### B. Create session 严格幂等链路

#### HIGH-B1: session 创建与 idempotency key 落库分离，存在双写窗口

- **模块**: `ve-server` API + idempotency store
- **位置**: `crates/ve-server/src/api/sessions.rs:232`, `crates/ve-server/src/api/sessions.rs:323`, `crates/ve-server/src/api/sessions.rs:351`, `crates/ve-server/src/db/idempotency.rs:124`
- **问题摘要**:
  - 当前是“先读 key -> 建 session -> 写 key”的分离流程
  - 并发相同 key 时，可能出现额外 session 已写入、但 key 最终只指向其中一个结果的情况

### C. SQLite 启动迁移重放链路

#### HIGH-C1: migration 006 会擦除 rerun provenance

- **模块**: SQLite migration
- **位置**: `crates/ve-server/src/db/migrations/sqlite/006_session_pending_status.sql:21`
- **问题摘要**:
  - `sessions` 表重建时没有保留 `rerun_from_session_id`
  - 对 rerun/resume 来说，这会破坏来源链和幂等闭环

#### MEDIUM-C2: migration 002 的逐步补丁重放仍可能留下部分应用状态

- **模块**: SQLite migration runner
- **位置**: `crates/ve-server/src/db/mod.rs:234`, `crates/ve-server/src/db/mod.rs:262`
- **问题摘要**:
  - migration 002 不是一次性 schema 收敛，而是逐语句补丁执行
  - 对中断恢复与状态一致性的保证仍偏弱

## 当前优先级建议

1. 先处理 `HIGH-01` / `HIGH-A1`，因为它直接影响 archived rerun 是否会返回一个并未真正 ready 的 live session。
2. 再处理 `HIGH-03` / `HIGH-C1`，因为它会在 SQLite migration replay 中破坏 rerun provenance。
3. `HIGH-02` / `HIGH-B1` 仍应保留在 ECC 未关闭项中，后续需要单独收敛 strict idempotency。
4. 两个 MEDIUM 项当前都更像“未完成的可靠性与验证收口”，不应在 HIGH 未关闭前被误判为完成。
