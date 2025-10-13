import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
  // Add this line to tell Vite where your HTML file is
  root: "src",

  // Configure multi-page app build
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "src/index-new.html"),
        schema: resolve(__dirname, "src/schema.html")
      }
    },
    // Copy additional assets
    assetsInclude: ['**/*.json', '**/*.css']
  },

  // prevent vite from clearing screen
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    hmr: {
      port: 1420,
    },
  },
});
