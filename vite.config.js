import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from 'vite';
import { isoImport } from 'vite-plugin-iso-import';

/** @type {import('vite').UserConfig} */
const config = defineConfig({
    plugins: [
        sveltekit(),
        isoImport()
    ],
    optimizeDeps: {
        include: ['js_reader'],
    },
    css: {
        preprocessorOptions: {
            scss: {
                additionalData: `
                  @import './src/sass/variables.scss';
                `,
            }
        },
    }
});

export default config;
