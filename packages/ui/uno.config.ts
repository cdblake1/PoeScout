import { defineConfig, presetUno, presetIcons } from "unocss";

export default defineConfig({
  presets: [presetUno(), presetIcons()],
  theme: {
    colors: {
      poe: {
        bg: "#0c0c0e",
        surface: "#1a1a1f",
        border: "#2a2a30",
        text: "#c8c8d0",
        muted: "#6b6b78",
        accent: "#c8a252",
        prefix: "#8888ff",
        suffix: "#88cc00",
        unique: "#af6025",
        rare: "#ffff77",
        magic: "#8888ff",
        normal: "#c8c8c8",
        currency: "#aa9e82",
        gem: "#1ba29b",
        fire: "#960000",
        cold: "#366492",
        lightning: "#ffd700",
      },
    },
  },
});
