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

/// 前端的轻量 scheme 提示。**这不是安全边界**——绕过界面直接 invoke 就没有它。
/// 权威校验长在 engine 的 `set_setting` 写入路径上（01-05）。两边都要有：
/// 这一层是体验（不必等一次 IPC 往返才知道写错了），那一层才是机制。
function looksLikeHttpUrl(raw: string): boolean {
  const trimmed = raw.trim();
  return trimmed.startsWith("http://") || trimmed.startsWith("https://");
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
    if (secretDraft.trim() === "") {
      setKeyNotice({ tone: "error", text: "请先填写密钥。" });
      return;
    }
    saveKey.mutate(secretDraft);
  }

  function submitUrl() {
    setUrlNotice(null);
    if (!looksLikeHttpUrl(urlDraft)) {
      setUrlNotice({ tone: "error", text: errorCopy("invalid_url") });
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
        <p>
          当前状态：
          <strong data-testid="api-key-status">
            {keyStatus.isPending ? "读取中…" : keyStatus.data ? "已配置" : "未配置"}
          </strong>
        </p>
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
        <p style={hint}>
          当前值：<code>{baseUrl.data ?? "（未设置）"}</code>
        </p>

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

function NoticeLine({ notice }: { notice: Notice }) {
  if (!notice) return null;
  if (notice.tone === "error") {
    return (
      <p role="alert" style={{ color: "#b00020" }}>
        {notice.text}
      </p>
    );
  }
  return <p style={{ color: "#00701a" }}>{notice.text}</p>;
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
