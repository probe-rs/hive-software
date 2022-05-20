import { fileURLToPath, URL } from "url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from "@vuetify/vite-plugin";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vuetify({
      autoImport: true,
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  server: {
    proxy: {
      "/graphql/backend": {
        target: "https://192.168.1.85:4356",
        changeOrigin: true,
        secure: false,
      },
      "/auth/backend": {
        target: "https://192.168.1.85:4356",
        changeOrigin: true,
        secure: false,
      },
    },
  },
});
