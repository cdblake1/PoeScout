import { Component } from "solid-js";

const KEYBINDS = [
  { key: "Ctrl+Q", action: "Capture hovered item → look up base" },
  { key: "F2", action: "Toggle overlay mode" },
  { key: "Esc", action: "Exit overlay (when in overlay)" },
];

const KeybindsPanel: Component<{ onClose: () => void }> = (props) => {
  return (
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <div class="bg-poe-surface border border-poe-border rounded-lg p-6 w-80 shadow-xl">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-poe-accent font-bold text-sm uppercase tracking-wider">
            Keybinds
          </h2>
          <button
            class="text-poe-muted hover:text-poe-text text-lg leading-none"
            onClick={() => props.onClose()}
          >
            x
          </button>
        </div>

        <table class="w-full text-sm">
          <tbody>
            {KEYBINDS.map((kb) => (
              <tr class="border-b border-poe-border/50 last:border-0">
                <td class="py-2 pr-4">
                  <kbd class="px-2 py-0.5 bg-poe-bg border border-poe-border rounded text-poe-accent text-xs font-mono">
                    {kb.key}
                  </kbd>
                </td>
                <td class="py-2 text-poe-text text-xs">{kb.action}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default KeybindsPanel;
