import React, { useContext } from "react";
import { getComponent } from "../registry/componentRegistry";
import { ComponentNodeSchema, type UIComponentNode } from "../schemas/uiSchema";
import { AppContext } from "../context/AppContext";

interface DynamicRendererProps {
  config: unknown;
  fallback?: unknown;
  viewId?: string;
}

const DynamicRenderer: React.FC<DynamicRendererProps> = ({ config, fallback, viewId }) => {
  const appContext = useContext(AppContext);
  const result = ComponentNodeSchema.safeParse(config);

  if (!result.success) {
    console.warn("Invalid component config:", result.error);

    if (fallback) {
      return <DynamicRenderer config={fallback} viewId={viewId} />;
    }

    return <div style={{ color: "red" }}>Invalid UI Configuration</div>;
  }

  const validatedConfig = result.data;
  const nodeProps = (validatedConfig.props ?? {}) as Record<string, unknown>;
  const isHidden = nodeProps.hidden === true || nodeProps.visible === false;
  if (isHidden) {
    return null;
  }
  const Component = getComponent(validatedConfig.type);

  if (!Component) {
    return <div style={{ color: "red" }}>No fallback component registered</div>;
  }

  const children = validatedConfig.children?.map((childConfig: UIComponentNode, index: number) => (
    <DynamicRenderer
      key={`${validatedConfig.type}-${index}`}
      config={childConfig}
      viewId={viewId}
    />
  ));

  return (
    <Component
      {...nodeProps}
      __nodeId={validatedConfig.nodeId}
      __viewId={viewId}
      __builder={validatedConfig.builder}
      __componentType={validatedConfig.type}
      type={validatedConfig.type}
      command={validatedConfig.command}
      context={appContext}
    >
      {children}
    </Component>
  );
};

export default DynamicRenderer;
