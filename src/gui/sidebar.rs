use dioxus::prelude::*;

use super::state::Page;

#[component]
pub fn Sidebar(
    current_page: Page,
    on_navigate: EventHandler<Page>,
    dark_mode: bool,
    on_toggle_dark: EventHandler<()>,
) -> Element {
    let version = env!("CARGO_PKG_VERSION");
    let toggle_label = if dark_mode { "Light Mode" } else { "Dark Mode" };
    let toggle_icon = if dark_mode { "\u{2600}" } else { "\u{263d}" };

    rsx! {
        nav { class: "sidebar",
            div {
                style: "padding: 16px 20px; margin-bottom: 8px;",
                span {
                    style: "font-size: 18px; font-weight: 700; color: var(--text-primary);",
                    "bashers"
                }
                span {
                    style: "font-size: 12px; color: var(--text-secondary); margin-left: 8px;",
                    "v{version}"
                }
            }
            div { style: "flex: 1;",
                for page in Page::all() {
                    {
                        let page = *page;
                        let active = page == current_page;
                        let class = if active { "nav-item active" } else { "nav-item" };
                        rsx! {
                            div {
                                class: "{class}",
                                onclick: move |_| on_navigate.call(page),
                                "{page.label()}"
                            }
                        }
                    }
                }
            }
            button {
                class: "theme-toggle",
                onclick: move |_| on_toggle_dark.call(()),
                span { "{toggle_icon}" }
                "{toggle_label}"
            }
        }
    }
}
