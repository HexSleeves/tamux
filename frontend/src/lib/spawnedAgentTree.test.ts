import { describe, expect, it } from "vitest";
import { deriveSpawnedAgentTree } from "./spawnedAgentTree.ts";

describe("deriveSpawnedAgentTree", () => {
  it("nests descendants by parent_task_id and keeps threadless nodes visible but closed", () => {
    const tree = deriveSpawnedAgentTree(
      [
        {
          id: "root-task",
          status: "in_progress",
          created_at: 10,
          thread_id: "thread-root",
        },
        {
          id: "child-task",
          status: "in_progress",
          created_at: 20,
          thread_id: "thread-child",
          parent_task_id: "root-task",
          parent_thread_id: "thread-root",
        },
        {
          id: "leaf-task",
          status: "completed",
          created_at: 30,
          parent_task_id: "child-task",
          parent_thread_id: "thread-child",
        },
      ],
      "thread-root",
    );

    expect(tree.root?.item.id).toBe("root-task");
    expect(tree.root?.openable).toBe(true);
    expect(tree.root?.live).toBe(true);
    expect(tree.root?.children[0]?.item.id).toBe("child-task");
    expect(tree.root?.children[0]?.openable).toBe(true);
    expect(tree.root?.children[0]?.live).toBe(true);
    expect(tree.root?.children[0]?.children[0]?.item.id).toBe("leaf-task");
    expect(tree.root?.children[0]?.children[0]?.openable).toBe(false);
    expect(tree.root?.children[0]?.children[0]?.live).toBe(false);
  });

  it("resolves the visible root from parent_thread_id when the active thread is the parent thread", () => {
    const tree = deriveSpawnedAgentTree(
      [
        {
          id: "unrelated",
          status: "completed",
          created_at: 5,
          thread_id: "thread-unrelated",
        },
        {
          id: "spawned-root",
          status: "in_progress",
          created_at: 15,
          thread_id: "thread-spawned",
          parent_thread_id: "thread-parent",
        },
      ],
      "thread-parent",
    );

    expect(tree.root?.item.id).toBe("spawned-root");
    expect(tree.root?.openable).toBe(true);
    expect(tree.root?.live).toBe(true);
  });

  it("keeps descendant nodes visible when an intermediate parent task is missing", () => {
    const tree = deriveSpawnedAgentTree(
      [
        {
          id: "root-task",
          status: "in_progress",
          created_at: 10,
          thread_id: "thread-root",
        },
        {
          id: "orphan-child",
          status: "in_progress",
          created_at: 20,
          thread_id: "thread-orphan",
          parent_task_id: "missing-parent",
          parent_thread_id: "thread-root",
        },
        {
          id: "grandchild",
          status: "completed",
          created_at: 30,
          thread_id: "thread-grandchild",
          parent_task_id: "orphan-child",
          parent_thread_id: "thread-orphan",
        },
      ],
      "thread-root",
    );

    expect(tree.root?.item.id).toBe("root-task");
    expect(tree.root?.children[0]?.item.id).toBe("orphan-child");
    expect(tree.root?.children[0]?.children[0]?.item.id).toBe("grandchild");
    expect(tree.root?.children[0]?.children[0]?.openable).toBe(true);
  });
});
