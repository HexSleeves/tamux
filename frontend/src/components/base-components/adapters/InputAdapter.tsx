import { Input } from "../../ui/Input";
import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { InputProps, ViewProps } from "../shared";
import { executeCommand } from "../../../registry/commandRegistry";
import { useViewBuilderStore } from "../../../lib/viewBuilderStore";

export function InputAdapter(props: ViewProps) {
  const { style, className, children, visible, hidden, builderMeta, componentProps } =
    splitTypedViewProps<InputProps>(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const {
    placeholder,
    type = "text",
    name,
    command,
    className: contentClassName,
    style: contentStyle,
  } = componentProps;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <Input
        type={type}
        placeholder={placeholder}
        name={name}
        className={contentClassName}
        style={contentStyle}
        onBlur={() => {
          if (!isEditMode && command) {
            void executeCommand(command);
          }
        }}
      />
    ),
  });
}
