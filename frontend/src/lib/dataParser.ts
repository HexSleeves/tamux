/**
 * Lightweight structured data detection and parsing utilities.
 * Used by DataTable and StaticLog to render terminal output as tables.
 */

export type ParsedData = {
  headers: string[];
  rows: string[][];
};

export type DetectedFormat = "csv" | "tsv" | "json" | "plain";

/**
 * Heuristic format detection for terminal output text.
 * Checks for JSON arrays, TSV, and CSV patterns.
 */
export function detectFormat(text: string): DetectedFormat {
  const trimmed = text.trim();
  if (!trimmed || trimmed.length < 4) return "plain";

  // JSON array detection
  if (trimmed.startsWith("[")) {
    try {
      const parsed = JSON.parse(trimmed);
      if (Array.isArray(parsed) && parsed.length > 0 && typeof parsed[0] === "object") {
        return "json";
      }
    } catch {
      // not valid JSON
    }
  }

  const lines = trimmed.split("\n").filter((l) => l.trim().length > 0);
  if (lines.length < 2) return "plain";

  // TSV detection: consistent tab-separated columns
  const tabCounts = lines.slice(0, 5).map((l) => (l.match(/\t/g) ?? []).length);
  if (tabCounts[0] >= 1 && tabCounts.every((c) => c === tabCounts[0])) {
    return "tsv";
  }

  // CSV detection: consistent comma-separated columns with header-like first line
  const commaCounts = lines.slice(0, 5).map((l) => countCSVFields(l));
  if (commaCounts[0] >= 2 && commaCounts.every((c) => c === commaCounts[0])) {
    return "csv";
  }

  return "plain";
}

/**
 * Count CSV fields in a line, respecting quoted fields.
 */
function countCSVFields(line: string): number {
  let count = 1;
  let inQuotes = false;
  for (let i = 0; i < line.length; i++) {
    const ch = line[i];
    if (ch === '"') {
      inQuotes = !inQuotes;
    } else if (ch === "," && !inQuotes) {
      count++;
    }
  }
  return count;
}

/**
 * Parse CSV text into headers and rows.
 * Handles quoted fields with embedded commas.
 */
export function parseCSV(text: string, delimiter = ","): ParsedData {
  const lines = text.trim().split("\n").filter((l) => l.trim().length > 0);
  if (lines.length === 0) return { headers: [], rows: [] };

  const parseLine = (line: string): string[] => {
    const fields: string[] = [];
    let current = "";
    let inQuotes = false;

    for (let i = 0; i < line.length; i++) {
      const ch = line[i];
      if (ch === '"') {
        if (inQuotes && i + 1 < line.length && line[i + 1] === '"') {
          current += '"';
          i++;
        } else {
          inQuotes = !inQuotes;
        }
      } else if (ch === delimiter && !inQuotes) {
        fields.push(current.trim());
        current = "";
      } else {
        current += ch;
      }
    }
    fields.push(current.trim());
    return fields;
  };

  const headers = parseLine(lines[0]);
  const rows = lines.slice(1).map(parseLine);
  return { headers, rows };
}

/**
 * Parse a JSON array of objects into headers and rows.
 */
export function parseJSON(text: string): ParsedData {
  try {
    const parsed = JSON.parse(text.trim());
    if (!Array.isArray(parsed) || parsed.length === 0) {
      return { headers: [], rows: [] };
    }

    // Collect all unique keys across all objects for headers
    const keySet = new Set<string>();
    for (const item of parsed) {
      if (typeof item === "object" && item !== null) {
        for (const key of Object.keys(item)) {
          keySet.add(key);
        }
      }
    }
    const headers = [...keySet];

    const rows = parsed.map((item) =>
      headers.map((header) => {
        const val = item?.[header];
        if (val === null || val === undefined) return "";
        if (typeof val === "object") return JSON.stringify(val);
        return String(val);
      })
    );

    return { headers, rows };
  } catch {
    return { headers: [], rows: [] };
  }
}

/**
 * Auto-detect format and parse structured text into a table.
 * Returns null if the text is not structured data.
 */
export function autoParse(text: string): ParsedData | null {
  const format = detectFormat(text);
  switch (format) {
    case "csv":
      return parseCSV(text, ",");
    case "tsv":
      return parseCSV(text, "\t");
    case "json":
      return parseJSON(text);
    default:
      return null;
  }
}
