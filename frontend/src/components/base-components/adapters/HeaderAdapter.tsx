import { CardDescription, CardTitle } from "../../ui/Card";
import { cn } from "../../ui/shared";
import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { HeaderProps, ViewProps } from "../shared";

export function HeaderAdapter(props: ViewProps) {
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
    componentProps,
  } = splitTypedViewProps<HeaderProps>(props);
  const { title, description, className: contentClassName, style: contentStyle } = componentProps;

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
    content: title || description ? (
      <div className={cn("grid gap-[var(--space-1)]", contentClassName)} style={contentStyle}>
        {title ? (
          <CardTitle className="m-0 text-[clamp(1.25rem,1rem+1vw,1.75rem)] leading-tight">{title}</CardTitle>
        ) : null}
        {description ? <CardDescription>{description}</CardDescription> : null}
      </div>
    ) : null,
  });
}
