import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { nodePolyfills } from "vite-plugin-node-polyfills";
import path from "path";
export default defineConfig({
  plugins: [
    react(),
    nodePolyfills({
      include: ["crypto", "buffer", "stream", "util", "process", "events", "string_decoder", "vm", "assert", "path"],
      globals: { Buffer: true, global: true, process: true },
      protocolImports: true,
    }),
  ],
  define: { "process.env": {} },
  resolve: { alias: { fs: path.resolve(__dirname, "src/fs-stub.ts") } },
});
