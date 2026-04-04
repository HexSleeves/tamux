const ERROR_ONLY_LOG_ENV = Object.freeze({
    TAMUX_LOG: 'error',
    AMUX_LOG: 'error',
    TAMUX_TUI_LOG: 'error',
    AMUX_GATEWAY_LOG: 'error',
    RUST_LOG: 'error',
});

function createChildLogEnv(baseEnv = process.env, options = {}) {
    const env = { ...baseEnv };
    if (!options.isPackaged) {
        return env;
    }

    return {
        ...env,
        ...ERROR_ONLY_LOG_ENV,
    };
}

module.exports = {
    ERROR_ONLY_LOG_ENV,
    createChildLogEnv,
};