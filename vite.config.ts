import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    outDir: "dist",
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    // Phase 1 只测量与呈报，不设 thresholds —— 理由与 Phase 2 的硬闸门交接见 01-02-SUMMARY.md
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      // 不写 include 时分母只含被测试导入过的文件，未测文件静默消失，
      // 数字会虚高到 100% —— 那样的"呈报"不如不报。
      include: ["src/**/*.{ts,tsx}"],
    },
  },
});
