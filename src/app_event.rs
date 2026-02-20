#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Push JS into the WebView
    EvalScript(String),
    /// Flash taskbar / dock when window not focused
    RequestAttention,
    /// Toggle always-on-top from the UI
    SetAlwaysOnTop(bool),
}
