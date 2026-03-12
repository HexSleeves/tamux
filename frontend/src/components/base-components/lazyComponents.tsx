import React, { Suspense } from "react";

export const AgentChatPanelLazy = React.lazy(() => import("../AgentChatPanel").then((module) => ({ default: module.AgentChatPanel })));
export const AgentChatPanelProviderLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelProvider })));
export const AgentChatPanelHeaderLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelHeader })));
export const AgentChatPanelTabsLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelTabs })));
export const AgentChatPanelCurrentSurfaceLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelCurrentSurface })));
export const AgentChatPanelThreadsSurfaceLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelThreadsSurface })));
export const AgentChatPanelChatSurfaceLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelChatSurface })));
export const AgentChatPanelTraceSurfaceLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelTraceSurface })));
export const AgentChatPanelUsageSurfaceLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelUsageSurface })));
export const AgentChatPanelContextSurfaceLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelContextSurface })));
export const AgentChatPanelGraphSurfaceLazy = React.lazy(() => import("../agent-chat-panel/runtime").then((module) => ({ default: module.AgentChatPanelGraphSurface })));
export const CommandHistoryPickerLazy = React.lazy(() => import("../CommandHistoryPicker").then((module) => ({ default: module.CommandHistoryPicker })));
export const CommandLogPanelLazy = React.lazy(() => import("../CommandLogPanel").then((module) => ({ default: module.CommandLogPanel })));
export const CommandPaletteLazy = React.lazy(() => import("../CommandPalette").then((module) => ({ default: module.CommandPalette })));
export const ExecutionCanvasLazy = React.lazy(() => import("../ExecutionCanvas").then((module) => ({ default: module.ExecutionCanvas })));
export const FileManagerPanelLazy = React.lazy(() => import("../FileManagerPanel").then((module) => ({ default: module.FileManagerPanel })));
export const NotificationPanelLazy = React.lazy(() => import("../NotificationPanel").then((module) => ({ default: module.NotificationPanel })));
export const SearchOverlayLazy = React.lazy(() => import("../SearchOverlay").then((module) => ({ default: module.SearchOverlay })));
export const SessionVaultPanelLazy = React.lazy(() => import("../SessionVaultPanel").then((module) => ({ default: module.SessionVaultPanel })));
export const SettingsPanelLazy = React.lazy(() => import("../SettingsPanel").then((module) => ({ default: module.SettingsPanel })));
export const SnippetPickerLazy = React.lazy(() => import("../SnippetPicker").then((module) => ({ default: module.SnippetPicker })));
export const SystemMonitorPanelLazy = React.lazy(() => import("../SystemMonitorPanel").then((module) => ({ default: module.SystemMonitorPanel })));
export const TimeTravelSliderLazy = React.lazy(() => import("../TimeTravelSlider").then((module) => ({ default: module.TimeTravelSlider })));
export const WebBrowserPanelLazy = React.lazy(() => import("../WebBrowserPanel").then((module) => ({ default: module.WebBrowserPanel })));

export const LazyView = ({ children }: { children: React.ReactNode }) => (
    <Suspense fallback={null}>{children}</Suspense>
);