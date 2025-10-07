import { defineConfig } from "vite";

export default defineConfig({
  // Add this line to tell Vite where your HTML file is
  root: "src",

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
