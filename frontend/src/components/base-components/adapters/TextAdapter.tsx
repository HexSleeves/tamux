import type React from "react";
import { cn } from "../../ui/shared";
import { renderEditableWrapper, splitTypedViewProps } from "../propUtils";
import type { TextProps, ViewProps } from "../shared";

function textClassNameForTag(tag: React.ElementType) {
  if (typeof tag !== "string") {
    return "text-[var(--text-sm)] text-[var(--text-primary)]";
  }

  if (/^h[1-6]$/.test(tag)) {
    return "font-semibold leading-tight text-[var(--text-primary)]";
  }

  if (tag === "p") {
    return "text-[var(--text-sm)] leading-6 text-[var(--text-secondary)]";
  }

  return "text-[var(--text-sm)] text-[var(--text-primary)]";
}

export function TextAdapter(props: ViewProps) {
  const { style, className, children, visible, hidden, builderMeta, componentProps } =
    splitTypedViewProps<TextProps>(props);
  const { value, as = "span", className: contentClassName, style: contentStyle } = componentProps;
  const Tag = as as React.ElementType;

  return renderEditableWrapper({
    style,
    className,
    children,
    visible,
    hidden,
    builderMeta,
    content: (
      <Tag className={cn(textClassNameForTag(Tag), contentClassName)} style={contentStyle}>
        {value}
      </Tag>
    ),
  });
}
