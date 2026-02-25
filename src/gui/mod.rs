#[cfg(feature = "gui")]
mod sidebar;
#[cfg(feature = "gui")]
mod pages;
pub(crate) mod state;
pub(crate) mod theme;
#[cfg(feature = "gui")]
pub(crate) mod server_fns;

#[cfg(feature = "gui")]
use dioxus::prelude::*;

#[cfg(feature = "gui")]
use state::Page;

#[cfg(feature = "gui")]
pub fn launch() {
    ensure_public_dir();
    dioxus::launch(app);
}

#[cfg(feature = "gui")]
fn ensure_public_dir() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));
    if let Some(dir) = exe_dir {
        let public = dir.join("public");
        if !public.exists() {
            let _ = std::fs::create_dir_all(&public);
            let _ = std::fs::write(
                public.join("index.html"),
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>bashers</title></head><body><div id=\"main\"></div></body></html>",
            );
        }
    }
}

#[cfg(feature = "gui")]
fn app() -> Element {
    let mut current_page = use_signal(Page::default);
    let mut dark_mode = use_signal(|| false);

    let root_class = if dark_mode() { "dark" } else { "" };

    rsx! {
        style { "{theme::global_css()}" }
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
        div { class: "{root_class}",
            sidebar::Sidebar {
                current_page: current_page(),
                on_navigate: move |page: Page| current_page.set(page),
                dark_mode: dark_mode(),
                on_toggle_dark: move |_| dark_mode.set(!dark_mode()),
            }
            main { class: "main-content",
                match current_page() {
                    Page::Show => rsx! { pages::show::ShowPage {} },
                    Page::Update => rsx! { pages::update::UpdatePage {} },
                    Page::Watch => rsx! { pages::watch::WatchPage {} },
                    Page::KubeTrack => rsx! { pages::kube_track::KubeTrackPage {} },
                }
            }
        }
    }
}
