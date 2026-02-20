/// Events sent from the backend (tokio) to the main thread (tao/wry).
/// The main thread calls `evaluate_script` to push data into the WebView.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Push a JS call into the WebView: `window.onRustMessage(json)`
    EvalScript(String),
}
