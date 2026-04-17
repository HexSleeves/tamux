import { isTaskTerminal, type AgentTaskStatus } from "./agentTaskQueue";
import type { SpawnedAgentTreeSource } from "./agentRuns";

export interface SpawnedAgentTreeContextItem {
    id: string;
    status: AgentTaskStatus;
    created_at: number;
    thread_id: string;
    isContextRoot: true;
}

export interface SpawnedAgentTreeNode<T extends SpawnedAgentTreeSource> {
    item: T | SpawnedAgentTreeContextItem;
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
    const anchor = pickLatest(items.filter((item) => item.thread_id === activeThreadId));
    const topLevelItems = anchor
        ? items.filter((item) => item.id === anchor.id)
        : sortByCreatedDesc(
            items.filter((item) => {
                if (item.parent_thread_id !== activeThreadId) {
                    return false;
                }
                return !item.parent_task_id || !byId.has(item.parent_task_id);
            }),
        );

    if (!anchor && topLevelItems.length === 0) {
        return { root: null };
    }

    const buildNode = (item: T, ancestry: Set<string>): SpawnedAgentTreeNode<T> => {
        const nextAncestry = new Set(ancestry);
        nextAncestry.add(item.id);

        const directChildren = items.filter((candidate) => candidate.parent_task_id === item.id);
        const fallbackChildren = item.thread_id
            ? items.filter((candidate) => {
                if (candidate.id === item.id) {
                    return false;
                }
                if (candidate.parent_thread_id !== item.thread_id) {
                    return false;
                }
                return !candidate.parent_task_id || !byId.has(candidate.parent_task_id);
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
        root: {
            item: anchor ?? {
                id: `${activeThreadId}-root`,
                status: anchor?.status ?? "in_progress",
                created_at: anchor?.created_at ?? Math.max(0, ...topLevelItems.map((item) => item.created_at)),
                thread_id: activeThreadId,
                isContextRoot: true,
            },
            children: anchor
                ? buildNode(anchor, new Set()).children
                : topLevelItems.map((candidate) => buildNode(candidate, new Set())),
            openable: Boolean(anchor?.thread_id),
            live: anchor ? !isTaskTerminal(anchor) : topLevelItems.some((item) => !isTaskTerminal(item)),
        },
    };
}
