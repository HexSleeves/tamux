import { TextArea } from "../../ui/TextArea";
import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { TextAreaProps, ViewProps } from "../shared";
import { executeCommand } from "../../../registry/commandRegistry";
import { useViewBuilderStore } from "../../../lib/viewBuilderStore";

export function TextAreaAdapter(props: ViewProps) {
  const { style, className, children, visible, hidden, builderMeta, componentProps } =
    splitTypedViewProps<TextAreaProps>(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const {
    placeholder,
    name,
    rows = 4,
    command,
    defaultValue,
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
      <TextArea
        placeholder={placeholder}
        name={name}
        rows={rows}
        defaultValue={defaultValue}
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
