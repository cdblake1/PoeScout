import "virtual:uno.css";
import "@unocss/reset/tailwind.css";
import { render } from "solid-js/web";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "./App";
import OverlayView from "./components/OverlayView";

const label = getCurrentWebviewWindow().label;

render(
  () => (label === "overlay" ? <OverlayView /> : <App />),
  document.getElementById("app")!,
);
