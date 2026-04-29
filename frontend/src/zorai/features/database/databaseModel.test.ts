import { describe, expect, it } from "vitest";
import { buildDatabaseRowUpdates, normalizeDatabasePageSize } from "./databaseModel";
import type { DatabaseTablePage } from "./databaseTypes";

const page: DatabaseTablePage = {
  tableName: "agent_messages",
  totalRows: 2,
  offset: 0,
  limit: 100,
  editable: true,
  columns: [
    { name: "id", declaredType: "INTEGER", nullable: false, primaryKey: true, editable: true },
    { name: "role", declaredType: "TEXT", nullable: true, primaryKey: false, editable: true },
    { name: "token_count", declaredType: "INTEGER", nullable: true, primaryKey: false, editable: true },
  ],
  rows: [
    { rowid: 1, values: { id: 1, role: "user", token_count: 12 } },
    { rowid: 2, values: { id: 2, role: "assistant", token_count: 24 } },
  ],
};

describe("databaseModel", () => {
  it("builds row updates with only changed values per column", () => {
    const updates = buildDatabaseRowUpdates(page, {
      "1:role": "operator",
      "1:token_count": "12",
      "2:token_count": "31",
    });

    expect(updates).toEqual([
      { rowid: 1, values: { role: "operator" } },
      { rowid: 2, values: { token_count: 31 } },
    ]);
  });

  it("keeps pagination page size bounded with a 100 row default", () => {
    expect(normalizeDatabasePageSize(undefined)).toBe(100);
    expect(normalizeDatabasePageSize(0)).toBe(1);
    expect(normalizeDatabasePageSize(9999)).toBe(500);
  });
});
