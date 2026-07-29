import { describe, expect, it, vi, beforeEach } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));

import { devPing, errorCopy } from "./ipc";

describe("devPing", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  it("invokes the dev_ping command", async () => {
    invoke.mockResolvedValue("3.53.2");
    await devPing();
    expect(invoke).toHaveBeenCalledWith("dev_ping");
  });

  it("returns the engine string unchanged", async () => {
    invoke.mockResolvedValue("3.53.2");
    await expect(devPing()).resolves.toBe("3.53.2");
  });

  it("propagates command errors instead of swallowing them", async () => {
    invoke.mockRejectedValue("sqlite error: disk I/O error");
    await expect(devPing()).rejects.toBe("sqlite error: disk I/O error");
  });
});

// ------------------------------------------------------------------ errorCopy
//
// 通用兜底的原文。写在这里而不是从 ipc.ts 导入，是为了让「兜底串被人改掉」这件事
// 表现为一条红测试，而不是两边一起漂走后仍然全绿。
const FALLBACK = "操作失败，请重试。";

/// 九个**正常短码**——它们是阴性对照。一个「一律返回兜底」的实现会让下面那批
/// 原型链输入全绿，只有这九条能把它照出来。
const KNOWN_CODES: ReadonlyArray<readonly [string, string]> = [
  ["invalid_url", "链接必须以 http:// 或 https:// 开头，并带有主机名。"],
  [
    "invalid_url_credentials",
    "端点链接里不能带用户名或密码（形如 user:pass@host），也不能带查询串（?…）或锚点（#…）。密钥请填在上面的 API key 栏——它只进系统钥匙串，不入数据库。",
  ],
  ["invalid_setting", "这个配置项不被接受（疑似密钥的键名一律不入库）。"],
  ["store_error", "写入本地数据库失败，请重试。"],
  ["secret_error", "系统钥匙串当前不可用，密钥没有保存。"],
  ["task_failed", "后台任务没能完成，请重试。"],
  ["channel_send_failed", "数据流通道已关闭。"],
  ["engine_error", "引擎遇到一个内部错误。"],
  ["listen_failed", "事件通道未能建立，界面不会随引擎变更自动刷新。"],
];

/// `Object.prototype` 上真实存在的成员名。用对象字面量做查找表时，
/// `TABLE["toString"]` 解析到的是**函数**而不是 `undefined`，`??` 因此不兜底。
const PROTOTYPE_MEMBERS = [
  "toString",
  "constructor",
  "valueOf",
  "hasOwnProperty",
  "__proto__",
  "isPrototypeOf",
  "propertyIsEnumerable",
  "toLocaleString",
] as const;

/// 非字符串输入。`errorCopy` 收的是 `unknown`——catch 到什么都可能。
const NON_STRING_INPUTS: ReadonlyArray<readonly [string, unknown]> = [
  ["number", 42],
  ["null", null],
  ["undefined", undefined],
  ["Error", new Error("x")],
  ["object", { code: "invalid_url" }],
  ["array", ["invalid_url"]],
];

describe("errorCopy", () => {
  it.each(KNOWN_CODES)("translates the %s short code", (code, expected) => {
    const copy = errorCopy(code);
    expect(typeof copy).toBe("string");
    expect(copy).toBe(expected);
  });

  it.each(PROTOTYPE_MEMBERS)(
    "returns the generic fallback string for the Object.prototype member %s",
    (member) => {
      const copy = errorCopy(member);
      // typeof 断言先行：回归时它给出的信息是「拿到的是个函数」，
      // 而不是一句难读的深比较失败。
      expect(typeof copy).toBe("string");
      expect(copy).toBe(FALLBACK);
    },
  );

  it("returns the generic fallback for an unrecognised short code", () => {
    expect(typeof errorCopy("nope")).toBe("string");
    expect(errorCopy("nope")).toBe(FALLBACK);
  });

  it.each(NON_STRING_INPUTS)("returns the generic fallback for a %s input", (_name, input) => {
    const copy = errorCopy(input);
    expect(typeof copy).toBe("string");
    expect(copy).toBe(FALLBACK);
  });

  // 声明类型与运行期行为一致这件事，只有把**全部**取样合起来看才成立：
  // 上面每一组单独看都可以被某个偷懒实现骗过（见 KNOWN_CODES 的注释）。
  it("never returns a non-string, for any sampled input", () => {
    const all: unknown[] = [
      ...KNOWN_CODES.map(([code]) => code),
      ...PROTOTYPE_MEMBERS,
      ...NON_STRING_INPUTS.map(([, input]) => input),
      "nope",
      "",
    ];
    for (const input of all) {
      expect(typeof errorCopy(input)).toBe("string");
    }
  });
});
