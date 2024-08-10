import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import { default as wasm } from "vite-plugin-wasm";

/** @type {import('vite').UserConfig} */
const config = defineConfig({
  plugins: [sveltekit(), wasm()],
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
