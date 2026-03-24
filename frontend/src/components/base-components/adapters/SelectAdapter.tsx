import { useMemo } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../ui/Select";
import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { SelectProps, ViewProps } from "../shared";
import { executeCommand } from "../../../registry/commandRegistry";
import { useViewBuilderStore } from "../../../lib/viewBuilderStore";

export function SelectAdapter(props: ViewProps) {
  const { style, className, children, visible, hidden, builderMeta, componentProps } =
    splitTypedViewProps<SelectProps>(props);
  const isEditMode = useViewBuilderStore((state) => state.isEditMode);
  const {
    name,
    value,
    options = [],
    command,
    placeholder,
    className: contentClassName,
    style: contentStyle,
  } = componentProps;
  const placeholderText = useMemo(() => {
    if (placeholder) {
      return placeholder;
    }

    return options.find((option) => option.value === value)?.label ?? options[0]?.label ?? "Select option";
  }, [options, placeholder, value]);

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <Select
        name={name}
        defaultValue={value ?? options[0]?.value}
        onValueChange={() => {
          if (!isEditMode && command) {
            void executeCommand(command);
          }
        }}
      >
        <SelectTrigger className={contentClassName} style={contentStyle}>
          <SelectValue placeholder={placeholderText} />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    ),
  });
}
