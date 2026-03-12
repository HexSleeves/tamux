import { registerComponent } from "./componentRegistry";
import * as BaseComponents from "../components/BaseComponents";

export const registerBaseComponents = (): void => {
  Object.entries(BaseComponents).forEach(([name, component]) => {
    if (name !== "UnknownComponent") {
      registerComponent(name, component);
    }
  });

  registerComponent("Unknown", BaseComponents.UnknownComponent);
};
