import type React from "react";

export type RegisteredComponent = React.ComponentType<any>;

export const ComponentRegistry = new Map<string, RegisteredComponent>();

export const registerComponent = (name: string, component: RegisteredComponent): void => {
  ComponentRegistry.set(name, component);
};

export const getComponent = (name: string): RegisteredComponent | undefined => {
  return ComponentRegistry.get(name) || ComponentRegistry.get("Unknown");
};

export const ComponentRegistryAPI = {
  register: registerComponent,
  get: getComponent,
  list: (): string[] => Array.from(ComponentRegistry.keys()),
};
