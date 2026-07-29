import { useEffect, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";

import {
  devEmitBusEvent,
  devSeedSampleDocs,
  devSmokeStream,
  errorCopy,
  searchDocuments,
  EVENT_CHANGED,
  LISTEN_FAILED,
  type EngineEvent,
  type SearchHit,
  type SmokeEvent,
} from "../lib/ipc";

/// 冒烟页固定的流长度。**1000 而不是个位数**：有序性只在"快速连发"下才可能被违反，
/// total=3 的流即使实现是乱序的也大概率看起来是对的——那种绿证明不了任何事。
const SMOKE_TOTAL = 1000;

/// 与 `Settings.tsx` 第 39 行**逐字同形**。两个页面各写一份是本 phase 刻意接受的
/// 重复（D-06 禁的是投机建共享布局层），但形状必须一致——否则下一个页面会出现第三种。
type Notice = { tone: "ok" | "error"; text: string } | null;

export type SmokeVerdict =
  | { ok: true; ticks: number }
  | { ok: false; reason: string };

/// 有序性校验。与 Rust 侧 `smoke::tests` 同口径：**逐位比较**而不是集合比较——
/// 「这些 seq 都出现过」在乱序时同样成立，而乱序正是这条通路唯一要防的失败模式。
export function verifySmokeStream(
  events: SmokeEvent[],
  total: number,
): SmokeVerdict {
  const first = events[0];
  if (!first || first.event !== "started") {
    return { ok: false, reason: "首条不是 Started" };
  }
  const last = events[events.length - 1];
  if (!last || last.event !== "finished") {
    return { ok: false, reason: "末条不是 Finished" };
  }

  const ticks = events.slice(1, -1);
  for (let i = 0; i < ticks.length; i++) {
    const ev = ticks[i];
    if (ev.event !== "tick") {
      return { ok: false, reason: `第 ${i} 条不是 Tick（实为 ${ev.event}）` };
    }
    if (ev.data.seq !== i) {
      return {
        ok: false,
        reason: `第 ${i} 条的 seq 是 ${ev.data.seq}，期望 ${i}（缺口或乱序）`,
      };
    }
  }
  if (ticks.length !== total) {
    return { ok: false, reason: `实收 ${ticks.length} 条 Tick，期望 ${total}` };
  }
  return { ok: true, ticks: ticks.length };
}

/// 隐藏 dev 冒烟页（D-06）：每条不可逆决策一个可点的验证入口。
///
/// 这是脚手架，后续 phase 逐步替换。**刻意不建正式布局/侧栏/文档树**——
/// Phase 2 才有导入功能，现在排布局是推测式设计。
export default function DevSmokePage() {
  const queryClient = useQueryClient();

  const [projectId, setProjectId] = useState("smoke-project");
  const [eventCount, setEventCount] = useState(0);
  const [verdict, setVerdict] = useState<SmokeVerdict | null>(null);
  const [streaming, setStreaming] = useState(false);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [notice, setNotice] = useState<Notice>(null);

  // 计数器走一条**独立**的 listen，与 useEngineInvalidation 的失效路径分开——
  // 这样「事件到了几次」是可直接读的数字，而不是要从 refetch 次数反推的东西。
  // cleanup 与 hook 侧同形：listen 返回 Promise<UnlistenFn>，漏掉它计数就会翻倍。
  //
  // **两个分支都必须接住 rejection。** `listen` 走插件命令 `plugin:event|listen`，受 ACL 管辖；
  // 而 `invoke('dev_emit_bus_event')` 是 generate_handler! 注册的自有命令，不过 ACL。
  // 于是 capability 缺失时发射照常成功、监听从未注册，计数停在 0 且毫无报错——
  // 因为 rejection 落进了没人接的 Promise。计数为 0 在正常状态下同样成立，指不出问题。
  useEffect(() => {
    let active = true;
    const pending = listen<EngineEvent>(EVENT_CHANGED, () => {
      setEventCount((n) => n + 1);
    });
    pending.catch(() => {
      if (active) setNotice({ tone: "error", text: errorCopy(LISTEN_FAILED) });
    });
    return () => {
      active = false;
      // 两参形式而不是 .then(un => un())：后者在 listen 失败时自己也是一个未处理的 rejection。
      pending.then(
        (unlisten) => unlisten(),
        () => {},
      );
    };
  }, []);

  // StrictMode 下 effect 双执行，两次挂载会各跑一遍——用 ref 记住"上一次收到的条数"
  // 没有意义，真正要看的是 cleanup 是否生效，那由计数器本身呈现。
  const collected = useRef<SmokeEvent[]>([]);

  async function handleEmit() {
    setNotice(null);
    try {
      await devEmitBusEvent(projectId, "d1");
    } catch (err) {
      setNotice({ tone: "error", text: errorCopy(err) });
    }
  }

  async function handleStream() {
    setNotice(null);
    setStreaming(true);
    setVerdict(null);
    collected.current = [];
    try {
      await devSmokeStream(SMOKE_TOTAL, (ev) => {
        collected.current.push(ev);
      });
      setVerdict(verifySmokeStream(collected.current, SMOKE_TOTAL));
    } catch (err) {
      setNotice({ tone: "error", text: errorCopy(err) });
    } finally {
      setStreaming(false);
    }
  }

  async function handleSeed() {
    setNotice(null);
    try {
      // 用引擎返回的 id，而不是页面上另存一份常量——两份必然漂移，
      // 而漂移的表现是「播种成功但搜不到」，正好长得像 FTS 坏了。
      setProjectId(await devSeedSampleDocs());
      setNotice({ tone: "ok", text: "样例文档已写入。" });
      void queryClient.invalidateQueries();
    } catch (err) {
      setNotice({ tone: "error", text: errorCopy(err) });
    }
  }

  async function handleSearch() {
    setNotice(null);
    try {
      setHits(await searchDocuments(projectId, query));
    } catch (err) {
      setNotice({ tone: "error", text: errorCopy(err) });
    }
  }

  return (
    <main style={page}>
      <h1>Dev 冒烟页</h1>
      <p style={hint}>
        隐藏页，不放导航入口。当前 project：<code>{projectId}</code>
      </p>
      <NoticeLine notice={notice} />

      <section style={card}>
        <h2>① 总线事件往返（notify-then-fetch）</h2>
        <p style={hint}>
          点一次应当只收到一次事件。反复进出本页之后再点，计数仍应与点击次数 1:1——
          计数翻倍就说明 listen 的 cleanup 漏了。
        </p>
        <p>
          收到事件次数：<strong data-testid="event-count">{eventCount}</strong>
        </p>
        {/*
          四个 handler 都是 `async`，而 `onClick` 的签名要求返回 `void`——直接
          `onClick={handleEmit}` 把一个 Promise 交给一个丢弃返回值的位置，正是
          `no-misused-promises` 守的形态（01-26 lint 闸门首跑点出的四处）。
          这四个 handler 内部各有完整 try/catch，所以 `void` 是准确的意思表达
          （「刻意不等它，且它不会 reject」），与本仓库已有的 `void
          queryClient.invalidateQueries()` 同一约定。
        */}
        <div style={row}>
          <button type="button" onClick={() => void handleEmit()}>
            触发总线事件
          </button>
          <button type="button" onClick={() => setEventCount(0)}>
            计数清零
          </button>
        </div>
      </section>

      <section style={card}>
        <h2>② Channel 有序流</h2>
        <p style={hint}>
          以 total={SMOKE_TOTAL} 收流，逐位断言第 i 条 Tick 的 seq 等于 i，并要求
          Started 在首、Finished 在末。
        </p>
        <div style={row}>
          <button
            type="button"
            onClick={() => void handleStream()}
            disabled={streaming}
          >
            Channel 有序流
          </button>
        </div>
        <p data-testid="stream-verdict">
          {verdict === null
            ? streaming
              ? "收流中…"
              : "尚未运行。"
            : verdict.ok
              ? `seq 校验通过 · 实收 ${verdict.ticks} 条`
              : `seq 校验失败：${verdict.reason}`}
        </p>
      </section>

      <section style={card}>
        <h2>③ 中文全文搜索（FTS5 trigram）</h2>
        <p style={hint}>
          空库上无论实现对错都返回 0，所以先插入样例文档再搜。「量子纠缠」是阴性对照——
          它必须返回 0，否则命中数说明不了任何事。
        </p>
        <div style={row}>
          <button type="button" onClick={() => void handleSeed()}>
            插入样例文档
          </button>
          <button type="button" onClick={() => setQuery("锚定引擎")}>
            填入「锚定引擎」
          </button>
          <button type="button" onClick={() => setQuery("量子纠缠")}>
            填入「量子纠缠」（阴性对照）
          </button>
        </div>

        <label htmlFor="smoke-query">搜索词</label>
        <input
          id="smoke-query"
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          style={input}
        />
        <div style={row}>
          <button type="button" onClick={() => void handleSearch()}>
            搜索
          </button>
        </div>

        <p>
          命中条数：
          <strong data-testid="search-count">
            {hits === null ? "—" : hits.length}
          </strong>
        </p>
        <ul>
          {(hits ?? []).slice(0, 5).map((hit) => (
            <li key={hit.docId}>
              {hit.title ?? "(无标题)"} — <code>{hit.relPath}</code>
            </li>
          ))}
        </ul>
      </section>
    </main>
  );
}

/// 与 `Settings.tsx` 的 `NoticeLine` **同源**（那边是第一份，本页是第二份）。
///
/// 分 tone 而不是一个 region 通吃：`status` 是礼貌 live region，读屏不会为它打断，
/// 而失败——尤其是「本页从此收不到任何事件」这种不可重试的失败——必须被播报出来。
/// 反过来，成功不走 `alert`：每次成功都打断，等于把 alert 这个信号磨没。
/// 无通知时两个 region 都不渲染，不留空节点。
function NoticeLine({ notice }: { notice: Notice }) {
  if (!notice) return null;
  if (notice.tone === "error") {
    return (
      <p role="alert" style={{ color: "#b00020" }}>
        {notice.text}
      </p>
    );
  }
  return (
    <p role="status" style={{ color: "#00701a" }}>
      {notice.text}
    </p>
  );
}

const page = {
  fontFamily: "system-ui, sans-serif",
  padding: "2rem",
  lineHeight: 1.6,
  maxWidth: "44rem",
} as const;

const card = {
  border: "1px solid #ddd",
  borderRadius: "8px",
  padding: "1rem 1.25rem",
  marginTop: "1.5rem",
} as const;

const hint = { color: "#555", fontSize: "0.9rem" } as const;
const input = { display: "block", width: "100%", margin: "0.35rem 0 0.75rem" } as const;
const row = { display: "flex", gap: "0.5rem", flexWrap: "wrap" } as const;
