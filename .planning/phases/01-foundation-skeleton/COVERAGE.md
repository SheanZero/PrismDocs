# API Coverage — MCP (Model Context Protocol, rmcp 2.2 server side)

> Full coverage by default. Opt-outs are explicit, reasoned decisions.
>
> Phase 1 的外部协议面只有一个：内嵌 loopback MCP server（D-07）。PrismDocs 在此
> 关系中是 **server**，能力面 = 对 agent 暴露的 tool 列表。Phase 1 的交付边界由
> `01-RESEARCH.md` § Pattern 4 明确写死：「prism-mcp 只需要一个能编译、能起 axum、
> 能通过一次注入 trait 返回假数据的最小 handler。工具面、端口发现、CLI helper
> 契约全部是 Phase 6。」下表把这条边界逐项记录为**已决定的 opt-out**，而不是
> 无人发现的空洞。
>
> prism-llm（OpenAI 兼容端点 / Anthropic Messages API）在 Phase 1 只建 crate 骨架
> 与密钥入口，不发出任何请求；其能力面矩阵属于 Phase 4（A3：传输层先行），届时
> 重新从全覆盖基线决定，不继承本表的任何 opt-out。

| capability | decision | reason |
|---|---|---|
| streamable-http transport (`StreamableHttpService` on axum loopback) | INTEGRATE | |
| session management (`LocalSessionManager`) | INTEGRATE | |
| server initialize / capabilities handshake | INTEGRATE | |
| bearer 鉴权中间件 | INTEGRATE | |
| Host + Origin allowlist 中间件（DNS-rebinding 防护） | INTEGRATE | |
| graceful shutdown (`CancellationToken`) | INTEGRATE | |
| tool: `list_feedback` | OPT-OUT | Phase 6 (F4 回流闭环) 范围；Phase 1 只用一个假数据 handler 证明注入通路（LOOP-02） |
| tool: `get_feedback` | OPT-OUT | Phase 6 (F4) 范围；同上 |
| tool: `respond_to_comment` | OPT-OUT | Phase 6 (F4) 范围；依赖 Phase 5 的评论状态机，Phase 1 尚无评论表 |
| tool: `get_document_comments` | OPT-OUT | Phase 6 (F4) 范围；同上 |
| tool: `get_context_pack` | OPT-OUT | Phase 7 (F7) 范围；D-09 的 trait 反转正是为了届时新增不动 prism-mcp |
| tool: `list_cards` | OPT-OUT | Phase 7 (F7) 范围；同上 |
| resources / prompts / sampling（MCP 协议其余三类原语） | OPT-OUT | 产品形态不需要——PrismDocs 对 agent 暴露的是动作型 tool，不是资源目录；无后续 phase 计划引入 |
| 端口发现与 CLI helper 契约（headersHelper / SessionStart hook） | OPT-OUT | Phase 6 范围（D-10 在 Phase 1 只建空占位 binary，使其依赖面进入 `cargo tree -d` 检查） |
| 客户端侧 MCP（PrismDocs 作为 MCP client 消费他人 server） | OPT-OUT | 不在产品范围内——PrismDocs 只做 server 侧 |
