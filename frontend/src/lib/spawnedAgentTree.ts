import { isTaskTerminal } from "./agentTaskQueue";
import type { SpawnedAgentTreeSource } from "./agentRuns";

export interface SpawnedAgentTreeNode<T extends SpawnedAgentTreeSource> {
    item: T;
    children: SpawnedAgentTreeNode<T>[];
    openable: boolean;
    live: boolean;
}

export interface SpawnedAgentTree<T extends SpawnedAgentTreeSource> {
    root: SpawnedAgentTreeNode<T> | null;
}

function pickLatest<T extends SpawnedAgentTreeSource>(items: readonly T[]): T | null {
    if (items.length === 0) {
        return null;
    }

    return items.reduce((latest, item) => {
        if (item.created_at !== latest.created_at) {
            return item.created_at > latest.created_at ? item : latest;
        }
        return item.id > latest.id ? item : latest;
    });
}

function sortByCreatedDesc<T extends SpawnedAgentTreeSource>(items: readonly T[]): T[] {
    return [...items].sort((left, right) => {
        if (left.created_at !== right.created_at) {
            return right.created_at - left.created_at;
        }
        return left.id.localeCompare(right.id);
    });
}

export function deriveSpawnedAgentTree<T extends SpawnedAgentTreeSource>(
    items: readonly T[],
    activeThreadId: string | null | undefined,
): SpawnedAgentTree<T> {
    if (!activeThreadId || items.length === 0) {
        return { root: null };
    }

    const byId = new Map(items.map((item) => [item.id, item] as const));
    const threadRoot = pickLatest(items.filter((item) => item.thread_id === activeThreadId));
    const parentThreadRoot = threadRoot ?? pickLatest(items.filter((item) => item.parent_thread_id === activeThreadId));

    if (!parentThreadRoot) {
        return { root: null };
    }

    const anchorThreadId = parentThreadRoot.thread_id ?? activeThreadId;

    const buildNode = (item: T, ancestry: Set<string>): SpawnedAgentTreeNode<T> => {
        const nextAncestry = new Set(ancestry);
        nextAncestry.add(item.id);

        const directChildren = items.filter((candidate) => candidate.parent_task_id === item.id);
        const fallbackChildren = item.thread_id
            ? items.filter((candidate) => {
                if (candidate.id === item.id) {
                    return false;
                }
                if (candidate.parent_task_id && byId.has(candidate.parent_task_id)) {
                    return false;
                }
                return candidate.parent_thread_id === item.thread_id;
            })
            : item.id === parentThreadRoot.id && anchorThreadId
                ? items.filter((candidate) => {
                    if (candidate.id === item.id) {
                        return false;
                    }
                    if (candidate.parent_task_id && byId.has(candidate.parent_task_id)) {
                        return false;
                    }
                    return candidate.parent_thread_id === anchorThreadId;
                })
                : [];

        const childCandidates = sortByCreatedDesc([
            ...directChildren,
            ...fallbackChildren,
        ]).filter((candidate, index, array) => array.findIndex((entry) => entry.id === candidate.id) === index);

        const children = childCandidates
            .filter((candidate) => !nextAncestry.has(candidate.id))
            .map((candidate) => buildNode(candidate, nextAncestry));

        return {
            item,
            children,
            openable: Boolean(item.thread_id),
            live: !isTaskTerminal(item),
        };
    };

    return {
        root: buildNode(parentThreadRoot, new Set()),
    };
}
