import React from "react";
import { AgentApprovalOverlay as AgentApprovalOverlayComponent } from "./AgentApprovalOverlay";
import { LayoutContainer as LayoutContainerComponent } from "./LayoutContainer";
import { Sidebar as SidebarComponent } from "./Sidebar";
import { StatusBar as StatusBarComponent } from "./StatusBar";
import { SurfaceTabBar as SurfaceTabBarComponent } from "./SurfaceTabBar";
import { TitleBar as TitleBarComponent } from "./TitleBar";
import {
  AgentChatPanelChatSurfaceLazy,
  AgentChatPanelContextSurfaceLazy,
  AgentChatPanelCurrentSurfaceLazy,
  AgentChatPanelGraphSurfaceLazy,
  AgentChatPanelHeaderLazy,
  AgentChatPanelLazy,
  AgentChatPanelProviderLazy,
  AgentChatPanelTabsLazy,
  AgentChatPanelThreadsSurfaceLazy,
  AgentChatPanelTraceSurfaceLazy,
  AgentChatPanelUsageSurfaceLazy,
  CommandHistoryPickerLazy,
  CommandLogPanelLazy,
  CommandPaletteLazy,
  ExecutionCanvasLazy,
  FileManagerPanelLazy,
  LazyView,
  NotificationPanelLazy,
  SearchOverlayLazy,
  SessionVaultPanelLazy,
  SettingsPanelLazy,
  SnippetPickerLazy,
  SystemMonitorPanelLazy,
  TimeTravelSliderLazy,
  WebBrowserPanelLazy,
} from "./base-components/lazyComponents";
import { MissionDeck } from "./base-components/MissionDeck";
import { renderEditableWrapper, splitViewProps } from "./base-components/propUtils";
import type {
  ButtonProps,
  HeaderProps,
  InputProps,
  SelectProps,
  SpacerProps,
  TextAreaProps,
  TextProps,
  UnknownProps,
  ViewProps,
} from "./base-components/shared";
import { executeCommand } from "../registry/commandRegistry";
import { useViewBuilderStore } from "../lib/viewBuilderStore";
import { useWorkspaceStore } from "../lib/workspaceStore";

export { AppRuntimeBridge } from "./base-components/AppRuntimeBridge";
export { MissionDeck } from "./base-components/MissionDeck";
export { ViewMount } from "./base-components/ViewMount";

export const Container: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: null,
  });
};

export const Header: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  const { title, description } = componentProps as HeaderProps;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <div>
        <h1>{title}</h1>
        {description ? <p>{description}</p> : null}
      </div>
    ),
  });
};

export const Text: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, builderMeta, componentProps } = splitViewProps(props);
  const { value, as = "span" } = componentProps as TextProps;
  const Tag = as as React.ElementType;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: <Tag>{value}</Tag>,
  });
};

export const Button: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, builderMeta, componentProps } = splitViewProps(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const { label, command, variant = "primary" } = componentProps as ButtonProps;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <button
        type="button"
        className={`btn-${variant}`}
        onClick={() => {
          if (!isEditMode && command) {
            void executeCommand(command);
          }
        }}
      >
        {label}
      </button>
    ),
  });
};

export const Input: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, builderMeta, componentProps } = splitViewProps(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const { placeholder, type = "text", name, command } = componentProps as InputProps;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <input
        type={type}
        placeholder={placeholder}
        name={name}
        onBlur={() => {
          if (!isEditMode && command) {
            void executeCommand(command);
          }
        }}
      />
    ),
  });
};

export const TextArea: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, builderMeta, componentProps } = splitViewProps(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const { placeholder, name, rows = 4, command, defaultValue } = componentProps as TextAreaProps;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <textarea
        placeholder={placeholder}
        name={name}
        rows={rows}
        defaultValue={defaultValue}
        onBlur={() => {
          if (!isEditMode && command) {
            void executeCommand(command);
          }
        }}
      />
    ),
  });
};

export const Select: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, builderMeta, componentProps } = splitViewProps(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const { name, value, options = [], command } = componentProps as SelectProps;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <select
        name={name}
        defaultValue={value}
        onChange={() => {
          if (!isEditMode && command) {
            void executeCommand(command);
          }
        }}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>{option.label}</option>
        ))}
      </select>
    ),
  });
};

export const Divider: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, builderMeta } = splitViewProps(props);

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: <div className={["amux-divider", "amux-divider--subtle"].concat(className ? [className] : []).join(" ")} />,
  });
};

export const Spacer: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, builderMeta, componentProps } = splitViewProps(props);
  const { size = 16 } = componentProps as SpacerProps;

  return renderEditableWrapper({
    style: {
      width: size,
      height: size,
      flexShrink: 0,
      ...(style ?? {}),
    },
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: null,
  });
};

export const UnknownComponent: React.FC<UnknownProps> = ({ type }) => (
  <div style={{ color: "red", border: "1px solid red", padding: "10px" }}>
    Unknown Component: {type ?? "(missing type)"}
  </div>
);


export const TitleBar: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <TitleBarComponent {...(componentProps as any)} />,
  });
};

export const SurfaceTabBar: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <SurfaceTabBarComponent {...(componentProps as any)} />,
  });
};

export const LayoutContainer: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LayoutContainerComponent {...(componentProps as any)} />,
  });
};

export const StatusBar: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <StatusBarComponent {...(componentProps as any)} />,
  });
};

export const Sidebar: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  const sidebarVisible = useWorkspaceStore((s) => s.sidebarVisible);
  if (!sidebarVisible) {
    return null;
  }

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <SidebarComponent {...(componentProps as any)} />,
  });
};


export const CommandPalette: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <CommandPaletteLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const NotificationPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <NotificationPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const SettingsPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <SettingsPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const SessionVaultPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <SessionVaultPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const CommandLogPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <CommandLogPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const CommandHistoryPicker: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <CommandHistoryPickerLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const SearchOverlay: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <SearchOverlayLazy
          {...(componentProps as any)}
          style={{
            position: "static",
            top: "auto",
            right: "auto",
            zIndex: "auto",
            ...(componentProps as any)?.style,
          }}
          className={className}
        />
      </LazyView>
    ),
  });
};

export const AgentChatPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <AgentChatPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const AgentChatPanelProvider: React.FC<ViewProps> = (props) => {
  const { style, className, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps, children } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <AgentChatPanelProviderLazy {...(componentProps as any)}>{children}</AgentChatPanelProviderLazy>
      </LazyView>
    ),
  });
};

export const AgentChatPanelHeader: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelHeaderLazy /></LazyView>,
  });
};

export const AgentChatPanelTabs: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelTabsLazy /></LazyView>,
  });
};

export const AgentChatPanelCurrentSurface: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelCurrentSurfaceLazy /></LazyView>,
  });
};

export const AgentChatPanelThreadsSurface: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelThreadsSurfaceLazy /></LazyView>,
  });
};

export const AgentChatPanelChatSurface: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelChatSurfaceLazy /></LazyView>,
  });
};

export const AgentChatPanelTraceSurface: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelTraceSurfaceLazy /></LazyView>,
  });
};

export const AgentChatPanelUsageSurface: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelUsageSurfaceLazy /></LazyView>,
  });
};

export const AgentChatPanelContextSurface: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelContextSurfaceLazy /></LazyView>,
  });
};

export const AgentChatPanelGraphSurface: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <LazyView><AgentChatPanelGraphSurfaceLazy /></LazyView>,
  });
};

export const SnippetPicker: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <SnippetPickerLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const SystemMonitorPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <SystemMonitorPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const FileManagerPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <FileManagerPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const TimeTravelSlider: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <TimeTravelSliderLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const ExecutionCanvas: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <ExecutionCanvasLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const WebBrowserPanel: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: (
      <LazyView>
        <WebBrowserPanelLazy {...(componentProps as any)} />
      </LazyView>
    ),
  });
};

export const AgentApprovalOverlay: React.FC<ViewProps> = (props) => {
  const { style, className, children, visible, hidden, resizable, resizeAxis, minWidth, minHeight, maxWidth, maxHeight, builderMeta, componentProps } = splitViewProps(props);
  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    resizable,
    resizeAxis,
    minWidth,
    minHeight,
    maxWidth,
    maxHeight,
    builderMeta,
    content: <AgentApprovalOverlayComponent {...(componentProps as any)} />,
  });
};

// Backward-compatible aliases for previously generated YAMLs.
export const TitleBarView = TitleBar;
export const SurfaceTabBarView = SurfaceTabBar;
export const LayoutContainerView = LayoutContainer;
export const StatusBarView = StatusBar;
export const SidebarView = Sidebar;
export const MissionDeckView = MissionDeck;
export const CommandPaletteView = CommandPalette;
export const NotificationPanelView = NotificationPanel;
export const SettingsPanelView = SettingsPanel;
export const SessionVaultPanelView = SessionVaultPanel;
export const CommandLogPanelView = CommandLogPanel;
export const CommandHistoryPickerView = CommandHistoryPicker;
export const SearchOverlayView = SearchOverlay;
export const AgentChatPanelView = AgentChatPanel;
export const AgentChatPanelProviderView = AgentChatPanelProvider;
export const AgentChatPanelHeaderView = AgentChatPanelHeader;
export const AgentChatPanelTabsView = AgentChatPanelTabs;
export const AgentChatPanelCurrentSurfaceView = AgentChatPanelCurrentSurface;
export const AgentChatPanelThreadsSurfaceView = AgentChatPanelThreadsSurface;
export const AgentChatPanelChatSurfaceView = AgentChatPanelChatSurface;
export const AgentChatPanelTraceSurfaceView = AgentChatPanelTraceSurface;
export const AgentChatPanelUsageSurfaceView = AgentChatPanelUsageSurface;
export const AgentChatPanelContextSurfaceView = AgentChatPanelContextSurface;
export const AgentChatPanelGraphSurfaceView = AgentChatPanelGraphSurface;
export const SnippetPickerView = SnippetPicker;
export const SystemMonitorPanelView = SystemMonitorPanel;
export const FileManagerPanelView = FileManagerPanel;
export const TimeTravelSliderView = TimeTravelSlider;
export const ExecutionCanvasView = ExecutionCanvas;
export const WebBrowserPanelView = WebBrowserPanel;
export const AgentApprovalOverlayView = AgentApprovalOverlay;
