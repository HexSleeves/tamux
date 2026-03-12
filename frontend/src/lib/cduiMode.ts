const STORAGE_KEY = "amux_feature_cdui";

export const isCDUIEnabled = (): boolean => {
  const query = new URLSearchParams(window.location.search);

  if (query.get("cdui") === "1" || query.get("ui") === "cdui") {
    return true;
  }

  if (query.get("cdui") === "0" || query.get("ui") === "classic") {
    return false;
  }

  return localStorage.getItem(STORAGE_KEY) === "1";
};

export const setCDUIEnabled = (enabled: boolean): void => {
  if (enabled) {
    localStorage.setItem(STORAGE_KEY, "1");
  } else {
    localStorage.removeItem(STORAGE_KEY);
  }

  const url = new URL(window.location.href);
  if (enabled) {
    url.searchParams.set("cdui", "1");
    if (url.searchParams.get("ui") === "classic") {
      url.searchParams.delete("ui");
    }
  } else {
    url.searchParams.delete("cdui");
    if (url.searchParams.get("ui") === "cdui") {
      url.searchParams.delete("ui");
    }
  }

  window.history.replaceState({}, "", `${url.pathname}${url.search}${url.hash}`);
};
