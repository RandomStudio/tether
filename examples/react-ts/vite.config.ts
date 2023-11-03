import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      Buffer: "buffer",
      mqtt: "mqtt/dist/mqtt.js",
    },
  },
});
