mod app_event;
mod backend;
mod db;
mod discovery;
mod ipc;
mod network;
mod protocol;
mod state;
mod ui;

use app_event::AppEvent;
use tao::event::{ElementState, Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::keyboard::{Key, ModifiersState};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

fn main() {
    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Gustavio")
        .with_inner_size(tao::dpi::LogicalSize::new(920.0, 580.0))
        .with_always_on_top(true)
        .build(&event_loop)
        .expect("Failed to build window");

    // Start backend
    let ipc_tx = backend::start(proxy);

    // Build WebView
    let ipc_tx_clone = ipc_tx.clone();
    let webview = WebViewBuilder::new()
        .with_html(ui::HTML)
        .with_ipc_handler(move |msg: wry::http::Request<String>| {
            let body = msg.body().clone();
            let _ = ipc_tx_clone.send(body);
        })
        .with_devtools(cfg!(debug_assertions))
        .build(&window)
        .expect("Failed to build WebView");

    let mut is_focused = true;
    let mut modifiers = ModifiersState::empty();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Focused(focused),
                ..
            } => {
                is_focused = focused;
            }
            Event::WindowEvent {
                event: WindowEvent::ModifiersChanged(mods),
                ..
            } => {
                modifiers = mods;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            tao::event::KeyEvent {
                                logical_key: Key::Character(ref ch),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // Ctrl+D (Windows/Linux) or Cmd+D (macOS) to minimize/hide
                let ctrl_or_cmd = if cfg!(target_os = "macos") {
                    modifiers.super_key()
                } else {
                    modifiers.control_key()
                };
                if ctrl_or_cmd && &**ch == "d" {
                    window.set_minimized(true);
                }
            }
            Event::UserEvent(AppEvent::EvalScript(ref js)) => {
                let _ = webview.evaluate_script(js);
            }
            Event::UserEvent(AppEvent::RequestAttention) => {
                if !is_focused {
                    window.request_user_attention(Some(
                        tao::window::UserAttentionType::Informational,
                    ));
                }
            }
            Event::UserEvent(AppEvent::SetAlwaysOnTop(on_top)) => {
                window.set_always_on_top(on_top);
            }
            _ => {}
        }
    });
}
