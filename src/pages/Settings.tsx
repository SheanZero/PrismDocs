import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import {
  apiKeyStatus,
  deleteApiKey,
  errorCopy,
  getSetting,
  setApiKey,
  setBaseUrl,
  SETTING_BASE_URL,
} from "../lib/ipc";

/// 前端的轻量端点提示。**这不是安全边界**——绕过界面直接 invoke 就没有它。
/// 权威校验长在 engine 的 `set_setting` 写入路径上（01-05 / 01-10）。两边都要有：
/// 这一层是体验（不必等一次 IPC 往返才知道写错了），那一层才是机制。
///
/// 这一层现在多认一种形态（链接里带凭据），是为了让用户**在按下保存之前**就知道
/// 那串东西不该填在这里——不是因为它变成了防线。
///
/// 判定项与 engine 侧 `validate_base_url` 逐项对齐（scheme / host / userinfo / query / fragment）。
/// **两个方向的分歧都要防**，而且更糟的是反方向那个：
///   - 前端放过、engine 拒绝 → 用户多等一次 IPC 往返才看到同一句错误，代价有限；
///   - 前端拒绝、engine 会接受 → 用户看到的是一句自相矛盾的话（输入 `HTTPS://…`
///     却被告知「链接必须以 http:// 或 https:// 开头」），他没有任何办法照做。
///
/// 所以判定**先解析再看结构**，不做任何字节级前缀比较：URL scheme 大小写不敏感，
/// `URL` 与 engine 侧的 `url` crate 都会把它小写化，而 `startsWith("https://")` 不会。
/// 顺带消掉了「允许哪些 scheme」这条知识在本函数里被写两遍。
/// 返回 `null` 表示本地看不出问题。
function localUrlIssue(raw: string): "invalid_url" | "invalid_url_credentials" | null {
  let url: URL;
  try {
    url = new URL(raw.trim());
  } catch {
    return "invalid_url";
  }
  // `protocol` 带尾随冒号且已小写化，与 engine 的 `url.scheme()` 同口径。
  if (url.protocol !== "http:" && url.protocol !== "https:") return "invalid_url";
  if (url.hostname === "") return "invalid_url";
  // 两个条件都要：`https://user@host/v1` 的 `password` 是空串，只看密码会漏。
  if (url.username !== "" || url.password !== "") return "invalid_url_credentials";
  if (url.search !== "" || url.hash !== "") return "invalid_url_credentials";
  return null;
}

type Notice = { tone: "ok" | "error"; text: string } | null;

/// 设置页（D-06 / D-16a）：**可跳过**。不配置密钥时应用照常启动，本页也照常可用——
/// 这里没有引导向导、没有不可关闭的弹层、没有把用户挡在别处的跳转。
export default function SettingsPage() {
  const queryClient = useQueryClient();

  const keyStatus = useQuery({ queryKey: ["apiKeyStatus"], queryFn: apiKeyStatus });
  const baseUrl = useQuery({
    queryKey: ["settings", SETTING_BASE_URL],
    queryFn: () => getSetting(SETTING_BASE_URL),
  });

  // 密钥只在这个 state 里停留到提交为止，提交成功立刻清空。
  // 它不进 query cache、不进 URL、不进日志、不进任何错误文本。
  const [secretDraft, setSecretDraft] = useState("");
  const [keyNotice, setKeyNotice] = useState<Notice>(null);

  const [urlDraft, setUrlDraft] = useState("");
  const [urlNotice, setUrlNotice] = useState<Notice>(null);

  const saveKey = useMutation({
    mutationFn: (secret: string) => setApiKey(secret),
    onSuccess: () => {
      setSecretDraft("");
      setKeyNotice({ tone: "ok", text: "密钥已保存到系统钥匙串。" });
      void queryClient.invalidateQueries({ queryKey: ["apiKeyStatus"] });
    },
    onError: (err) => setKeyNotice({ tone: "error", text: errorCopy(err) }),
  });

  const removeKey = useMutation({
    mutationFn: () => deleteApiKey(),
    onSuccess: () => {
      setKeyNotice({ tone: "ok", text: "密钥已从系统钥匙串删除。" });
      void queryClient.invalidateQueries({ queryKey: ["apiKeyStatus"] });
    },
    onError: (err) => setKeyNotice({ tone: "error", text: errorCopy(err) }),
  });

  const saveUrl = useMutation({
    mutationFn: (url: string) => setBaseUrl(url),
    onSuccess: () => {
      setUrlNotice({ tone: "ok", text: "端点已保存。" });
      void queryClient.invalidateQueries({ queryKey: ["settings", SETTING_BASE_URL] });
    },
    onError: (err) => setUrlNotice({ tone: "error", text: errorCopy(err) }),
  });

  function submitKey() {
    setKeyNotice(null);
    // 判空与提交用**同一份值**。「只用 trim 判空却存原值」会造出一个看起来配置好了
    // 但永远用不了的凭据：从供应商控制台复制的密钥常带尾随换行，它被原样存进钥匙串，
    // `api_key_status()` 随后报「已配置」，而每次调用返回 401。
    // engine 侧 `prism_llm::secrets::set_api_key` 也做同一份裁剪（两端都做）。
    //
    // 局部变量刻意**不**取名 `secret`：`scripts/check-secrets.sh` 的关键词分支会把
    // `secret = <16 字符以上的裸值>` 判成提交进仓库的明文密钥。撞车时改代码不改防线
    // （scripts/check-secrets.sh 文件头的单向约定）。
    const trimmed = secretDraft.trim();
    if (trimmed === "") {
      setKeyNotice({ tone: "error", text: "请先填写密钥。" });
      return;
    }
    saveKey.mutate(trimmed);
  }

  function submitUrl() {
    setUrlNotice(null);
    const issue = localUrlIssue(urlDraft);
    if (issue !== null) {
      setUrlNotice({ tone: "error", text: errorCopy(issue) });
      return;
    }
    saveUrl.mutate(urlDraft.trim());
  }

  return (
    <main style={page}>
      <h1>设置</h1>
      <p style={hint}>
        不配置 LLM 密钥也可以正常使用应用的其余部分——本页随时可以留到以后再填。
      </p>

      <section style={card}>
        <h2>LLM API key</h2>
        {/*
          显式四态：pending / error / 有值 / 无值。**「读失败」不得落进「未配置」**——
          两者在界面上完全同形，而用户对「未配置」的反应是重新输入密钥；那次保存也会
          失败（钥匙串本来就不可用），且他仍然不知道原因。被拒的查询在 TanStack Query
          里以 `isPending === false` + `data === undefined` 落定，正好长成「没配置」。
        */}
        <p>
          当前状态：
          <strong data-testid="api-key-status">
            {keyStatus.isPending
              ? "读取中…"
              : keyStatus.isError
                ? "读取失败"
                : keyStatus.data
                  ? "已配置"
                  : "未配置"}
          </strong>
        </p>
        <NoticeLine
          notice={
            keyStatus.isError
              ? { tone: "error", text: errorCopy(keyStatus.error) }
              : null
          }
        />
        <p style={hint}>
          密钥存进 macOS 钥匙串（service <code>PrismDocs</code> / account{" "}
          <code>llm_api_key</code>），不入数据库；本页只显示配置状态，任何时候都不回显原文。
        </p>

        <label htmlFor="api-key">API key</label>
        <input
          id="api-key"
          type="password"
          autoComplete="off"
          spellCheck={false}
          value={secretDraft}
          onChange={(e) => setSecretDraft(e.target.value)}
          style={input}
        />

        <div style={row}>
          <button type="button" onClick={submitKey} disabled={saveKey.isPending}>
            保存密钥
          </button>
          <button
            type="button"
            onClick={() => {
              setKeyNotice(null);
              removeKey.mutate();
            }}
            disabled={removeKey.isPending}
          >
            删除密钥
          </button>
        </div>

        <NoticeLine notice={keyNotice} />
      </section>

      <section style={card}>
        <h2>LLM 端点</h2>
        {/* 同上：读失败**不得**落到「（未设置）」那一支。真的没设置与读不出来，
            用户要做的事不一样，而两句话在界面上原本一模一样。 */}
        <p style={hint}>
          当前值：
          <code>
            {baseUrl.isPending
              ? "读取中…"
              : baseUrl.isError
                ? "读取失败"
                : (baseUrl.data ?? "（未设置）")}
          </code>
        </p>
        <NoticeLine
          notice={
            baseUrl.isError ? { tone: "error", text: errorCopy(baseUrl.error) } : null
          }
        />

        <label htmlFor="base-url">LLM 端点（base_url）</label>
        <input
          id="base-url"
          type="text"
          placeholder="https://api.anthropic.com"
          value={urlDraft}
          onChange={(e) => setUrlDraft(e.target.value)}
          style={input}
        />

        <div style={row}>
          <button type="button" onClick={submitUrl} disabled={saveUrl.isPending}>
            保存端点
          </button>
        </div>

        <NoticeLine notice={urlNotice} />
      </section>
    </main>
  );
}

/// 与 `DevSmoke.tsx` 的同名组件同源（01-22）：两个页面各写一份是本 phase 刻意接受的
/// 重复，D-06 禁的是投机建共享布局层。
///
/// 两个 tone 落在**不同的 live region** 上：error → `alert`（打断读屏，用户必须知道），
/// ok → `status`（等读屏读完当前内容再播报）。颜色不是可访问的通道——只靠 `#00701a`
/// 的话，读屏用户对「已保存」一无所知。
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
const row = { display: "flex", gap: "0.5rem" } as const;
