-- migration 001 — schema v1
--
-- 已发布的迁移不可修改。后续 phase 只 append 新文件（002、003…），不动本文件。
--
-- 形态选型（Phase 1 checkpoint 定案 · 方案 A）：
--   * external-content FTS5：内容只存一份，索引由三触发器强制同步，写路径不可能"忘记"同步
--   * documents.rowid_pk 为显式 INTEGER PRIMARY KEY：VACUUM 只重编号没有显式整型主键的表，
--     绑隐式 rowid 会让 FTS 索引与内容表静默错位（搜到错文档且不报错）
--   * 索引粒度保持 SQLite 默认的全粒度（本文件刻意不声明该选项）：降粒度会禁掉长度超过
--     3 个 unicode 字符的全文查询，D-01 的 4 字中文词（如「锚定引擎」）当场失效
--   * tokenize = trigram（D-01）：中英文统一走 substring 匹配，天然覆盖 CJK 混排；
--     case_sensitive 与 remove_diacritics 一律不声明、保持默认（不设 remove_diacritics
--     才保留 FTS5 对 GLOB/LIKE 的索引化能力，留作后续 phase 的余地）
--   * 三张真实表带 STRICT：类型错误在写入时报错而非静默转换。虚拟表不支持 STRICT
--
-- D-04 的最小集边界严格执行：不建 document_versions / blocks / comments / cards——
-- 那些属于 Phase 2/3/5 各自的迁移，在真实需求出现前不做推测式设计。
--
-- 本文件不含任何连接级设置语句（journal_mode 等属于连接打开流程，见 open.rs 的六步序）。

CREATE TABLE projects (
  rowid_pk    INTEGER PRIMARY KEY,          -- 显式 rowid，VACUUM 安全
  id          TEXT    NOT NULL UNIQUE,      -- ULID，D-13 的 project-id
  name        TEXT    NOT NULL,
  root_path   TEXT    NOT NULL,
  created_at  INTEGER NOT NULL
) STRICT;

CREATE TABLE documents (
  rowid_pk     INTEGER PRIMARY KEY,         -- FTS 的 content_rowid 绑定这一列
  id           TEXT    NOT NULL UNIQUE,     -- ULID
  project_id   TEXT    NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  rel_path     TEXT    NOT NULL,
  title        TEXT,
  content      TEXT    NOT NULL,            -- D-04：含内容列供 FTS 验证
  content_hash TEXT    NOT NULL,            -- blake3
  updated_at   INTEGER NOT NULL,
  UNIQUE(project_id, rel_path)
) STRICT;

CREATE VIRTUAL TABLE documents_fts USING fts5(
  title,
  content,
  content       = 'documents',
  content_rowid = 'rowid_pk',
  tokenize      = 'trigram'
);

-- external content 表的索引同步是调用方的责任；官方给出的做法就是这三个触发器。
-- 把它交给数据库，而不是交给"每条写路径都记得手工 INSERT INTO documents_fts"。
CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
  INSERT INTO documents_fts(rowid, title, content)
    VALUES (new.rowid_pk, new.title, new.content);
END;

CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
  INSERT INTO documents_fts(documents_fts, rowid, title, content)
    VALUES ('delete', old.rowid_pk, old.title, old.content);
END;

CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
  INSERT INTO documents_fts(documents_fts, rowid, title, content)
    VALUES ('delete', old.rowid_pk, old.title, old.content);
  INSERT INTO documents_fts(rowid, title, content)
    VALUES (new.rowid_pk, new.title, new.content);
END;

CREATE TABLE settings (
  key         TEXT    PRIMARY KEY,          -- D-05：非密钥配置（base_url、模型标识…）
  value       TEXT    NOT NULL,             -- 密钥绝不入库，只进钥匙串
  updated_at  INTEGER NOT NULL
) STRICT;
