import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { SpacerProps, ViewProps } from "../shared";

export function SpacerAdapter(props: ViewProps) {
  const { style, className, children, visible, hidden, builderMeta, componentProps } =
    splitTypedViewProps<SpacerProps>(props);
  const { size = 16 } = componentProps;

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
}
