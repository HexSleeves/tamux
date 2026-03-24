//! Plugin API proxy error types and request structure.
//!
//! Provides `PluginApiError` (structured error enum for all API proxy failures)
//! and `RenderedRequest` (output of template rendering, input to HTTP execution).

/// Structured error type for plugin API proxy operations.
/// Each variant produces an actionable error message for the agent/user.
#[derive(Debug, thiserror::Error)]
pub enum PluginApiError {
    #[error("SSRF blocked: request to {url} targets an internal/private IP range")]
    SsrfBlocked { url: String },

    #[error("Rate limited: plugin '{plugin}' exceeded rate limit. Retry after {retry_after_secs}s")]
    RateLimited {
        plugin: String,
        retry_after_secs: u64,
    },

    #[error("Template error: {detail}")]
    TemplateError { detail: String },

    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },

    #[error("Request timed out (30s limit)")]
    Timeout,

    #[error("Endpoint '{endpoint}' not found in plugin '{plugin}'")]
    EndpointNotFound { plugin: String, endpoint: String },

    #[error("Plugin '{name}' not found or not loaded")]
    PluginNotFound { name: String },

    #[error("Plugin '{name}' is disabled")]
    PluginDisabled { name: String },
}

/// Output of template rendering: a fully-resolved HTTP request ready to execute.
#[derive(Debug, Clone)]
pub struct RenderedRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssrf_blocked_display_contains_url() {
        let err = PluginApiError::SsrfBlocked {
            url: "http://127.0.0.1/secret".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("127.0.0.1/secret"), "msg: {msg}");
    }

    #[test]
    fn rate_limited_display_contains_retry_info() {
        let err = PluginApiError::RateLimited {
            plugin: "gmail".to_string(),
            retry_after_secs: 30,
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("gmail"), "msg: {msg}");
        assert!(msg.contains("30"), "msg: {msg}");
    }

    #[test]
    fn template_error_display() {
        let err = PluginApiError::TemplateError {
            detail: "missing variable".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("missing variable"), "msg: {msg}");
    }

    #[test]
    fn http_error_display() {
        let err = PluginApiError::HttpError {
            status: 404,
            body: "not found".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("404"), "msg: {msg}");
        assert!(msg.contains("not found"), "msg: {msg}");
    }

    #[test]
    fn timeout_display() {
        let err = PluginApiError::Timeout;
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("30s"), "msg: {msg}");
    }

    #[test]
    fn endpoint_not_found_display() {
        let err = PluginApiError::EndpointNotFound {
            plugin: "gmail".to_string(),
            endpoint: "send_email".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("send_email"), "msg: {msg}");
        assert!(msg.contains("gmail"), "msg: {msg}");
    }

    #[test]
    fn plugin_not_found_display_contains_name() {
        let err = PluginApiError::PluginNotFound {
            name: "missing-plugin".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("missing-plugin"), "msg: {msg}");
    }

    #[test]
    fn plugin_disabled_display() {
        let err = PluginApiError::PluginDisabled {
            name: "disabled-plugin".to_string(),
        };
        let msg = err.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("disabled-plugin"), "msg: {msg}");
    }
}
