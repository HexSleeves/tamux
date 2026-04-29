import type { DatabaseRowUpdate, DatabaseTablePage } from "./databaseTypes";

export const DEFAULT_DATABASE_PAGE_SIZE = 100;
export const MAX_DATABASE_PAGE_SIZE = 500;

export function normalizeDatabasePageSize(value: number | undefined): number {
  if (!Number.isFinite(value)) return DEFAULT_DATABASE_PAGE_SIZE;
  return Math.min(MAX_DATABASE_PAGE_SIZE, Math.max(1, Math.trunc(value ?? DEFAULT_DATABASE_PAGE_SIZE)));
}

export function databaseDraftKey(rowid: number, columnName: string): string {
  return `${rowid}:${columnName}`;
}

export function displayDatabaseValue(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (isBlobPlaceholder(value)) return `<BLOB ${value.bytes} bytes>`;
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}

export function isBlobPlaceholder(value: unknown): value is { type: "blob"; bytes: number; preview?: string } {
  return Boolean(
    value
    && typeof value === "object"
    && (value as { type?: unknown }).type === "blob"
    && typeof (value as { bytes?: unknown }).bytes === "number",
  );
}

export function parseDatabaseDraftValue(originalValue: unknown, draftValue: string): unknown {
  if (originalValue === null) return draftValue.trim() === "" ? null : draftValue;
  if (typeof originalValue === "number") {
    const next = Number(draftValue);
    return Number.isFinite(next) ? next : draftValue;
  }
  if (typeof originalValue === "boolean") {
    const normalized = draftValue.trim().toLowerCase();
    if (normalized === "true" || normalized === "1") return true;
    if (normalized === "false" || normalized === "0") return false;
  }
  return draftValue;
}

export function buildDatabaseRowUpdates(
  page: DatabaseTablePage | null,
  drafts: Record<string, string>,
): DatabaseRowUpdate[] {
  if (!page?.editable) return [];
  const updatesByRow = new Map<number, Record<string, unknown>>();
  for (const row of page.rows) {
    if (typeof row.rowid !== "number") continue;
    for (const column of page.columns) {
      if (!column.editable) continue;
      const originalValue = row.values[column.name];
      if (isBlobPlaceholder(originalValue)) continue;
      const key = databaseDraftKey(row.rowid, column.name);
      if (!Object.prototype.hasOwnProperty.call(drafts, key)) continue;
      const parsedValue = parseDatabaseDraftValue(originalValue, drafts[key]);
      if (JSON.stringify(parsedValue) === JSON.stringify(originalValue)) continue;
      const rowUpdate = updatesByRow.get(row.rowid) ?? {};
      rowUpdate[column.name] = parsedValue;
      updatesByRow.set(row.rowid, rowUpdate);
    }
  }
  return Array.from(updatesByRow.entries()).map(([rowid, values]) => ({ rowid, values }));
}
