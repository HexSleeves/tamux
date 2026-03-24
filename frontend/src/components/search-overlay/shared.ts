import type { CSSProperties } from "react";
import type { TerminalSearchOptions } from "../../lib/terminalRegistry";

export type SearchOverlayProps = {
    style?: CSSProperties;
    className?: string;
};

export type SearchActions = {
  activePaneId: string | null;
  query: string;
  searchOpts: TerminalSearchOptions;
    setQuery: (value: string) => void;
    doSearch: (query: string, opts?: TerminalSearchOptions) => void;
    findNext: () => void;
    findPrev: () => void;
    toggle: () => void;
    clearAndClose: () => void;
    caseSensitive: boolean;
  useRegex: boolean;
  setCaseSensitive: React.Dispatch<React.SetStateAction<boolean>>;
  setUseRegex: React.Dispatch<React.SetStateAction<boolean>>;
};
