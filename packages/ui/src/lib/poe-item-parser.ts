export interface ParsedPoEItem {
  baseName: string | null;
  itemClass: string | null;
  rarity: string | null;
  itemLevel: number | null;
}

export function parsePoEItem(raw: string): ParsedPoEItem {
  const lines = raw.split(/\r?\n/);
  let rarity: string | null = null;
  let itemClass: string | null = null;
  let baseName: string | null = null;
  let itemLevel: number | null = null;

  let rarityLineIndex = -1;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();

    if (line.startsWith("Item Class:")) {
      itemClass = line.replace("Item Class:", "").trim();
    }

    if (line.startsWith("Rarity:")) {
      rarity = line.replace("Rarity:", "").trim();
      rarityLineIndex = i;
    }

    const ilvlMatch = line.match(/^item\s*level:\s*(\d+)/i);
    if (ilvlMatch) {
      const parsed = parseInt(ilvlMatch[1], 10);
      if (!isNaN(parsed)) itemLevel = parsed;
    }
  }

  if (rarityLineIndex >= 0 && rarity) {
    if (rarity === "Normal" || rarity === "Currency" || rarity === "Gem") {
      // Base name is the line immediately after Rarity
      baseName = lines[rarityLineIndex + 1]?.trim() || null;
    } else if (rarity === "Magic") {
      // Magic: full name with prefixes/suffixes — return it for substring matching
      baseName = lines[rarityLineIndex + 1]?.trim() || null;
    } else {
      // Rare / Unique: skip random name, base is 2 lines after Rarity
      baseName = lines[rarityLineIndex + 2]?.trim() || null;
    }
  }

  // Strip separator lines
  if (baseName && baseName.startsWith("--------")) {
    baseName = null;
  }

  return { baseName, itemClass, rarity, itemLevel };
}
