import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  optimizeDeps: {
    include: ["tether-agent"],
  },
  build: {
    commonjsOptions: {
      include: [/tether-agent/, /node_modules/],
    },
  },
  resolve: {
    alias: {
      Buffer: "buffer",
      mqtt: "mqtt/dist/mqtt.js",
    },
  },
});
