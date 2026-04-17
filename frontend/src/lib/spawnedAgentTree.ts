import { isTaskTerminal } from "./agentTaskQueue";
import type { SpawnedAgentTreeSource } from "./agentRuns";

export interface SpawnedAgentTreeNode<T extends SpawnedAgentTreeSource> {
    item: T;
    children: SpawnedAgentTreeNode<T>[];
    openable: boolean;
    live: boolean;
}

export interface SpawnedAgentTree<T extends SpawnedAgentTreeSource> {
    activeThreadId: string;
    anchor: SpawnedAgentTreeNode<T> | null;
    roots: SpawnedAgentTreeNode<T>[];
}

function getTaskIdentity<T extends SpawnedAgentTreeSource>(item: T): string {
    return item.task_id ?? item.id;
}

function compareSpawnedAgentTreeItems<T extends SpawnedAgentTreeSource>(left: T, right: T): number {
    if (left.created_at !== right.created_at) {
        return right.created_at - left.created_at;
    }

    const leftIdentity = getTaskIdentity(left);
    const rightIdentity = getTaskIdentity(right);
    if (leftIdentity !== rightIdentity) {
        return leftIdentity.localeCompare(rightIdentity);
    }

    return left.id.localeCompare(right.id);
}

function uniqueByTaskIdentity<T extends SpawnedAgentTreeSource>(items: readonly T[]): T[] {
    const seen = new Set<string>();
    const result: T[] = [];
    for (const item of items) {
        const identity = getTaskIdentity(item);
        if (seen.has(identity)) {
            continue;
        }
        seen.add(identity);
        result.push(item);
    }
    return result;
}

function sortByCreatedDesc<T extends SpawnedAgentTreeSource>(items: readonly T[]): T[] {
    return uniqueByTaskIdentity(items).sort(compareSpawnedAgentTreeItems);
}

function pickLatest<T extends SpawnedAgentTreeSource>(items: readonly T[]): T | null {
    return sortByCreatedDesc(items)[0] ?? null;
}

function isVisibleRootCandidate<T extends SpawnedAgentTreeSource>(
    item: T,
    activeThreadId: string,
    identityLookup: ReadonlySet<string>,
): boolean {
    const hasResolvedParent = item.parent_task_id ? identityLookup.has(item.parent_task_id) : false;
    return Boolean(
        !hasResolvedParent &&
            (item.thread_id === activeThreadId || item.parent_thread_id === activeThreadId),
    );
}

function buildTreeNode<T extends SpawnedAgentTreeSource>(
    item: T,
    items: readonly T[],
    identityLookup: ReadonlySet<string>,
    rootIdentityLookup: ReadonlySet<string>,
    ancestry: Set<string>,
): SpawnedAgentTreeNode<T> {
    const currentIdentity = getTaskIdentity(item);
    const nextAncestry = new Set(ancestry);
    nextAncestry.add(currentIdentity);

    const directChildren = items.filter((candidate) => candidate.parent_task_id === currentIdentity);
    const fallbackChildren = item.thread_id
        ? items.filter((candidate) => {
            if (candidate.id === item.id) {
                return false;
            }
            if (rootIdentityLookup.has(getTaskIdentity(candidate))) {
                return false;
            }
            if (candidate.parent_task_id && identityLookup.has(candidate.parent_task_id)) {
                return false;
            }
            return candidate.parent_thread_id === item.thread_id;
        })
        : [];

    const childCandidates = sortByCreatedDesc([
        ...directChildren,
        ...fallbackChildren,
    ]).filter((candidate, index, array) => array.findIndex((entry) => getTaskIdentity(entry) === getTaskIdentity(candidate)) === index);

    return {
        item,
        children: childCandidates
            .filter((candidate) => !nextAncestry.has(getTaskIdentity(candidate)))
            .map((candidate) => buildTreeNode(candidate, items, identityLookup, rootIdentityLookup, nextAncestry)),
        openable: Boolean(item.thread_id),
        live: !isTaskTerminal(item),
    };
}

export function deriveSpawnedAgentTree<T extends SpawnedAgentTreeSource>(
    items: readonly T[],
    activeThreadId: string | null | undefined,
): SpawnedAgentTree<T> | null {
    if (!activeThreadId || items.length === 0) {
        return null;
    }

    const identityLookup = new Set(items.map((item) => getTaskIdentity(item)));
    const anchor = pickLatest(items.filter((item) => item.thread_id === activeThreadId));
    const visibleRootCandidates = sortByCreatedDesc(
        items.filter((item) => isVisibleRootCandidate(item, activeThreadId, identityLookup)),
    );
    const rootIdentityLookup = new Set(visibleRootCandidates.map((item) => getTaskIdentity(item)));

    const rootItems = visibleRootCandidates.filter((item) => !anchor || getTaskIdentity(item) !== getTaskIdentity(anchor));

    if (rootItems.length === 0 && !anchor) {
        return null;
    }

    return {
        activeThreadId,
        anchor: anchor
            ? buildTreeNode(anchor, items, identityLookup, rootIdentityLookup, new Set())
            : null,
        roots: rootItems.map((item) => buildTreeNode(item, items, identityLookup, rootIdentityLookup, new Set())),
    };
}
