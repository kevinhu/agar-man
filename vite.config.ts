import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import wasmPack from "vite-plugin-wasm-pack";
// https://vitejs.dev/config/
export default defineConfig({
    plugins: [wasmPack("./agar-man"), react()],
    build: {
        outDir: './dist',
        manifest: true,
        cssCodeSplit: false,
        sourcemap: true,
        cleanCssOptions: {
            sourceMap: true,
        },
    },
})
