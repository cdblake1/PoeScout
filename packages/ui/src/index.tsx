import "virtual:uno.css";
import "@unocss/reset/tailwind.css";
import { render } from "solid-js/web";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import App from "./App";
import OverlayView from "./components/OverlayView";
import TimerOverlay from "./components/maps/TimerOverlay";

const label = getCurrentWebviewWindow().label;

function Root() {
  if (label === "overlay") return <OverlayView />;
  if (label === "timer") return <TimerOverlay />;
  return <App />;
}

render(() => <Root />, document.getElementById("app")!);
