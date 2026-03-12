import type { UIViewBlockDefinition } from "../schemas/uiSchema";

export const VIEW_BUILDER_PRIMITIVE_BLOCK_IDS = new Set<string>([
    "amux-primitive-box",
    "amux-primitive-row",
    "amux-primitive-column",
    "amux-primitive-text",
    "amux-primitive-button",
    "amux-primitive-input",
    "amux-primitive-textarea",
    "amux-primitive-select",
    "amux-primitive-divider",
    "amux-primitive-spacer",
    "amux-primitive-header",
]);

export interface ViewBuilderPaletteItem {
    id: string;
    label: string;
    category: "primitive" | "component";
    description?: string;
    componentType?: string;
    blockId?: string;
}

const primitiveBlock = (
    title: string,
    layout: UIViewBlockDefinition["layout"],
    defaults?: UIViewBlockDefinition["defaults"],
): UIViewBlockDefinition => ({
    title,
    layout,
    ...(defaults ? { defaults } : {}),
    builder: {
        category: "primitive",
        editable: true,
    },
});

export const VIEW_BUILDER_PRIMITIVE_BLOCKS: Record<string, UIViewBlockDefinition> = {
    "amux-primitive-box": primitiveBlock(
        "Box",
        {
            id: "primitive-box-root",
            type: "Container",
            builder: { editable: true, droppable: true },
        },
        {
            visible: true,
            style: {
                display: "block",
                minWidth: 0,
                minHeight: 0,
            },
        },
    ),
    "amux-primitive-row": primitiveBlock(
        "Row",
        {
            id: "primitive-row-root",
            type: "Container",
            builder: { editable: true, droppable: true },
        },
        {
            visible: true,
            style: {
                display: "flex",
                flexDirection: "row",
                alignItems: "center",
                gap: 12,
                minWidth: 0,
                minHeight: 0,
            },
        },
    ),
    "amux-primitive-column": primitiveBlock(
        "Column",
        {
            id: "primitive-column-root",
            type: "Container",
            builder: { editable: true, droppable: true },
        },
        {
            visible: true,
            style: {
                display: "flex",
                flexDirection: "column",
                gap: 12,
                minWidth: 0,
                minHeight: 0,
            },
        },
    ),
    "amux-primitive-text": primitiveBlock(
        "Text",
        {
            id: "primitive-text-root",
            type: "Text",
            builder: { editable: true },
        },
        {
            visible: true,
            value: "Text",
        },
    ),
    "amux-primitive-button": primitiveBlock(
        "Button",
        {
            id: "primitive-button-root",
            type: "Button",
            builder: { editable: true },
        },
        {
            visible: true,
            label: "Button",
            variant: "primary",
        },
    ),
    "amux-primitive-input": primitiveBlock(
        "Input",
        {
            id: "primitive-input-root",
            type: "Input",
            builder: { editable: true },
        },
        {
            visible: true,
            placeholder: "Type here",
            type: "text",
        },
    ),
    "amux-primitive-textarea": primitiveBlock(
        "Textarea",
        {
            id: "primitive-textarea-root",
            type: "TextArea",
            builder: { editable: true },
        },
        {
            visible: true,
            placeholder: "Write something",
            rows: 4,
        },
    ),
    "amux-primitive-select": primitiveBlock(
        "Select",
        {
            id: "primitive-select-root",
            type: "Select",
            builder: { editable: true },
        },
        {
            visible: true,
            options: [
                { label: "Option A", value: "a" },
                { label: "Option B", value: "b" },
            ],
            value: "a",
        },
    ),
    "amux-primitive-divider": primitiveBlock(
        "Divider",
        {
            id: "primitive-divider-root",
            type: "Divider",
            builder: { editable: true },
        },
        {
            visible: true,
        },
    ),
    "amux-primitive-spacer": primitiveBlock(
        "Spacer",
        {
            id: "primitive-spacer-root",
            type: "Spacer",
            builder: { editable: true },
        },
        {
            visible: true,
            size: 16,
        },
    ),
    "amux-primitive-header": primitiveBlock(
        "Header",
        {
            id: "primitive-header-root",
            type: "Header",
            builder: { editable: true },
        },
        {
            visible: true,
            title: "Heading",
            description: "Supporting copy",
        },
    ),
};

export const VIEW_BUILDER_PRIMITIVE_PALETTE: ViewBuilderPaletteItem[] = [
    { id: "amux-primitive-box", label: "Box", category: "primitive", description: "Generic container", blockId: "amux-primitive-box" },
    { id: "amux-primitive-row", label: "Row", category: "primitive", description: "Horizontal stack", blockId: "amux-primitive-row" },
    { id: "amux-primitive-column", label: "Column", category: "primitive", description: "Vertical stack", blockId: "amux-primitive-column" },
    { id: "amux-primitive-text", label: "Text", category: "primitive", description: "Inline copy", blockId: "amux-primitive-text" },
    { id: "amux-primitive-header", label: "Header", category: "primitive", description: "Heading with description", blockId: "amux-primitive-header" },
    { id: "amux-primitive-button", label: "Button", category: "primitive", description: "Clickable action", blockId: "amux-primitive-button" },
    { id: "amux-primitive-input", label: "Input", category: "primitive", description: "Single-line field", blockId: "amux-primitive-input" },
    { id: "amux-primitive-textarea", label: "Textarea", category: "primitive", description: "Multi-line field", blockId: "amux-primitive-textarea" },
    { id: "amux-primitive-select", label: "Select", category: "primitive", description: "Dropdown field", blockId: "amux-primitive-select" },
    { id: "amux-primitive-divider", label: "Divider", category: "primitive", description: "Visual separator", blockId: "amux-primitive-divider" },
    { id: "amux-primitive-spacer", label: "Spacer", category: "primitive", description: "Empty spacing block", blockId: "amux-primitive-spacer" },
];

export const mergeViewBuilderPrimitiveBlocks = (
    blocks?: Record<string, UIViewBlockDefinition>,
): Record<string, UIViewBlockDefinition> => ({
    ...VIEW_BUILDER_PRIMITIVE_BLOCKS,
    ...(blocks ?? {}),
});