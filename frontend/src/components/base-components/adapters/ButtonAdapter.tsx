import { Button } from "../../ui/Button";
import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { ButtonProps, ViewProps } from "../shared";
import { executeCommand } from "../../../registry/commandRegistry";
import { useViewBuilderStore } from "../../../lib/viewBuilderStore";

function mapButtonVariant(variant?: ButtonProps["variant"]) {
  switch (variant) {
    case "secondary":
      return "secondary" as const;
    case "danger":
      return "destructive" as const;
    default:
      return "primary" as const;
  }
}

export function ButtonAdapter(props: ViewProps) {
  const { style, className, children, visible, hidden, builderMeta, componentProps } =
    splitTypedViewProps<ButtonProps>(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const { label, command, variant = "primary", className: contentClassName, style: contentStyle } = componentProps;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <Button
        variant={mapButtonVariant(variant)}
        className={contentClassName}
        style={contentStyle}
        onClick={() => {
          if (!isEditMode && command) {
            void executeCommand(command);
          }
        }}
      >
        {label}
      </Button>
    ),
  });
}
