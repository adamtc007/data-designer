import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
  // Configure TypeScript-first build
  root: "src",

  // Configure build output
  build: {
    outDir: "../dist",
    rollupOptions: {
      input: "main.ts"
    },
    // Copy additional assets
    assetsInclude: ['**/*.json', '**/*.css'],
    // Ensure TypeScript is properly processed
    target: 'es2020',
    minify: false, // Disable minification for debugging
    lib: {
      entry: "main.ts",
      name: "DataDesignerIDE",
      formats: ["es"]
    }
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
