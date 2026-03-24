import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { ViewProps } from "../shared";

export function ContainerAdapter(props: ViewProps) {
  const {
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
  } = splitTypedViewProps(props);

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
}
