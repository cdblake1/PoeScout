export function formatMs(ms: number): string {
  return ms < 1 ? "<1ms" : `${Math.round(ms)}ms`;
}

export function formatStatRange(min: number, max: number): string {
  if (min === max) return String(min);
  return `${min} to ${max}`;
}

export function generationLabel(type: string): string {
  switch (type) {
    case "prefix":
      return "Prefix";
    case "suffix":
      return "Suffix";
    case "corrupted":
      return "Corrupt";
    case "unique":
      return "Unique";
    default:
      return type;
  }
}

export function generationColor(type: string): string {
  switch (type) {
    case "prefix":
      return "text-poe-prefix";
    case "suffix":
      return "text-poe-suffix";
    default:
      return "text-poe-muted";
  }
}
