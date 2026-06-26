import { defineConfig } from "astro/config";

export default defineConfig({
  output: "static",
  build: {
    inlineStylesheets: "always",
  },
  vite: {
    build: {
      assetsInlineLimit: 1_000_000,
    },
  },
});
