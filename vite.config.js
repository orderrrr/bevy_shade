import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import { default as wasm } from "vite-plugin-wasm";
import topLevelAwait from 'vite-plugin-top-level-await';

/** @type {import('vite').UserConfig} */
const config = defineConfig({
    optimizeDeps: {
        exclude: ["brotli-wasm", "brotli-wasm/pkg.bundler/brotli_wasm_bg.wasm"],
    },
    plugins: [sveltekit(), wasm(), topLevelAwait()],
    css: {
        preprocessorOptions: {
            scss: {
                additionalData: `@import './src/sass/variables.scss';`,
            },
        },
    },
    server: {
        fs: {
            allow: ["./assets"],
        },
    },
});

export default config;
