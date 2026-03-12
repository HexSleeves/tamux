import type { CSSProperties } from "react";
import { useWorkspaceStore } from "../lib/workspaceStore";
import { useNotificationStore } from "../lib/notificationStore";
import { NotificationHeader } from "./notification-panel/NotificationHeader";
import { NotificationList } from "./notification-panel/NotificationList";

/**
 * Slide-over notification panel (Ctrl+I).
 * Shows notification history with mark-read and clear actions.
 */
type NotificationPanelProps = {
  style?: CSSProperties;
  className?: string;
};

export function NotificationPanel({ style, className }: NotificationPanelProps = {}) {
  const open = useWorkspaceStore((s) => s.notificationPanelOpen);
  const toggle = useWorkspaceStore((s) => s.toggleNotificationPanel);
  const notifications = useNotificationStore((s) => s.notifications);
  const markRead = useNotificationStore((s) => s.markRead);
  const markAllRead = useNotificationStore((s) => s.markAllRead);
  const clearAll = useNotificationStore((s) => s.clearAll);

  if (!open) return null;

  const unread = notifications.filter((n) => !n.isRead);

  return (
    <div
      onClick={toggle}
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(3,8,14,0.56)",
        zIndex: 900,
        display: "flex",
        justifyContent: "flex-end",
        backdropFilter: "none",
        ...(style ?? {}),
      }}
      className={className}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: 440,
          maxWidth: "90vw",
          height: "100%",
          background: "var(--bg-primary)",
          borderLeft: "1px solid var(--glass-border)",
          display: "flex",
          flexDirection: "column",
          boxShadow: "none",
        }}
      >
        <NotificationHeader
          unreadCount={unread.length}
          totalCount={notifications.length}
          markAllRead={markAllRead}
          clearAll={clearAll}
          close={toggle}
        />

        <NotificationList notifications={notifications} markRead={markRead} />
      </div>
    </div>
  );
}
