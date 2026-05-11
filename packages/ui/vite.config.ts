import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import UnoCSS from "unocss/vite";

export default defineConfig({
  plugins: [UnoCSS(), solid()],
  server: {
    port: 3000,
    strictPort: true,
  },
  build: {
    target: "esnext",
  },
});
