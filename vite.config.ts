import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  // pass your local crate path to the plugin
  plugins: [wasmPack("./agar-man"), react()],
});
