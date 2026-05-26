import { Component, For, Show } from "solid-js";

interface SparklineProps {
  data: number[];
  /** Internal viewBox width (rects scale to fill the container via SVG). */
  width?: number;
  height?: number;
  color?: string;
  label?: string;
}

/**
 * Minimal bar sparkline rendered as inline SVG (no chart-lib dependency).
 * The SVG sets `width="100%"` and uses `preserveAspectRatio="none"` so the bars
 * stretch to fill the parent column at any width.
 */
const Sparkline: Component<SparklineProps> = (props) => {
  const w = () => props.width ?? 240;
  const h = () => props.height ?? 40;
  const color = () => props.color ?? "#5fb3ff";

  const peak = () => Math.max(1, ...props.data);
  const barW = () => w() / Math.max(1, props.data.length);

  return (
    <div>
      <Show when={props.label}>
        <div class="text-poe-muted text-xs mb-1">{props.label}</div>
      </Show>
      <Show
        when={props.data.length > 0}
        fallback={
          <div
            class="text-poe-muted text-xs italic flex items-center"
            style={{ height: `${h()}px` }}
          >
            no data
          </div>
        }
      >
        <svg
          width="100%"
          height={h()}
          viewBox={`0 0 ${w()} ${h()}`}
          preserveAspectRatio="none"
          class="block"
        >
          <For each={props.data}>
            {(v, i) => {
              const bh = Math.max(1, (v / peak()) * (h() - 2));
              return (
                <rect
                  x={i() * barW()}
                  y={h() - bh}
                  width={Math.max(1, barW() - 1)}
                  height={bh}
                  fill={color()}
                  opacity={0.85}
                />
              );
            }}
          </For>
        </svg>
      </Show>
    </div>
  );
};

export default Sparkline;
