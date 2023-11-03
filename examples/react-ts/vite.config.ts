import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  // optimizeDeps: {
  //   exclude: ["tether-agent"],
  // },
  // build: {
  //   rollupOptions: {
  //     external: ["tether-agent"],
  //   },
  // },
  // resolve: {
  //   dedupe: ["tether-agent"],
  // },
});
