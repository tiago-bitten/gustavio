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
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

fn main() {
    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Gustavio Chat")
        .with_inner_size(tao::dpi::LogicalSize::new(1000.0, 700.0))
        .build(&event_loop)
        .expect("Failed to build window");

    // Start backend (tokio runtime on a background thread)
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

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(AppEvent::EvalScript(js)) => {
                let _ = webview.evaluate_script(&js);
            }
            _ => {}
        }
    });
}
