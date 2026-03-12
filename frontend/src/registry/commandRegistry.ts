export type CommandAction = (payload?: unknown) => unknown;

export const CommandRegistry = new Map<string, CommandAction>();

export const registerCommand = (commandId: string, actionFn: CommandAction): void => {
  CommandRegistry.set(commandId, actionFn);
};

export const executeCommand = (commandId: string, payload?: unknown): unknown => {
  const action = CommandRegistry.get(commandId);
  if (action) {
    return action(payload);
  }

  console.warn(`Command ${commandId} not found.`);
  return null;
};

export const CommandRegistryAPI = {
  register: registerCommand,
  execute: executeCommand,
  list: (): string[] => Array.from(CommandRegistry.keys()),
};
