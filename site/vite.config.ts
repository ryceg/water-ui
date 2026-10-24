import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

// GitHub Pages serves the project site under /<repo>/.
export default defineConfig({
  base: process.env.SITE_BASE ?? '/water-ui/',
  plugins: [tailwindcss(), svelte()],
  build: { target: 'esnext', assetsInlineLimit: 0 },
});
