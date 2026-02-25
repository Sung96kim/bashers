use std::collections::HashMap;

use dioxus::prelude::*;

use crate::gui::server_fns::list_packages;
use crate::utils::project::ProjectType;

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum PkgStatus {
    Idle,
    Updating,
    Done(String),
    Error(String),
}

#[component]
pub fn UpdatePage() -> Element {
    let mut selected = use_signal(Vec::<String>::new);
    let mut search = use_signal(String::new);
    let statuses = use_signal(|| HashMap::<String, PkgStatus>::new());

    let resource = use_resource(|| async { list_packages().await });

    let mut toggle_select = move |pkg: String| {
        let mut sel = selected();
        if sel.contains(&pkg) {
            sel.retain(|s| s != &pkg);
        } else {
            sel.push(pkg);
        }
        selected.set(sel);
    };

    let select_all = {
        let resource = resource.clone();
        move |_| {
            if let Some(Ok((_, ref pkgs))) = *resource.read() {
                let search_val = search().to_lowercase();
                let current: Vec<String> = if search_val.is_empty() {
                    pkgs.clone()
                } else {
                    pkgs.iter()
                        .filter(|p| p.to_lowercase().contains(&search_val))
                        .cloned()
                        .collect()
                };
                selected.set(current);
            }
        }
    };

    let deselect_all = move |_| {
        selected.set(vec![]);
    };

    rsx! {
        div {
            match &*resource.read() {
                None => rsx! {
                    div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 16px;",
                        h2 { "Update" }
                    }
                    p { style: "color: var(--text-secondary);", "Loading packages..." }
                },
                Some(Err(err)) => rsx! {
                    div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 16px;",
                        h2 { "Update" }
                    }
                    div { class: "error-banner", "Error: {err}" }
                },
                Some(Ok((pt, all_packages))) => {
                    let filtered: Vec<String> = {
                        let search_val = search().to_lowercase();
                        if search_val.is_empty() {
                            all_packages.clone()
                        } else {
                            all_packages
                                .iter()
                                .filter(|p| p.to_lowercase().contains(&search_val))
                                .cloned()
                                .collect()
                        }
                    };

                    rsx! {
                        div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 16px;",
                            h2 { "Update" }
                            span {
                                class: match pt {
                                    ProjectType::Uv => "badge badge-uv",
                                    ProjectType::Poetry => "badge badge-poetry",
                                    ProjectType::Cargo => "badge badge-cargo",
                                },
                                match pt {
                                    ProjectType::Uv => "Uv",
                                    ProjectType::Poetry => "Poetry",
                                    ProjectType::Cargo => "Cargo",
                                }
                            }
                        }

                        div { style: "margin-bottom: 16px; display: flex; gap: 12px;",
                            div { style: "flex: 1;",
                                input {
                                    placeholder: "Filter packages...",
                                    value: "{search}",
                                    oninput: move |e| search.set(e.value()),
                                }
                            }
                            button { class: "btn btn-secondary", onclick: select_all, "Select All" }
                            button { class: "btn btn-secondary", onclick: deselect_all, "Deselect All" }
                        }

                        div { class: "card",
                            table {
                                thead {
                                    tr {
                                        th { style: "width: 40px;", "" }
                                        th { "Package" }
                                        th { "Status" }
                                    }
                                }
                                tbody {
                                    for pkg in filtered.iter() {
                                        {
                                            let pkg_name = pkg.clone();
                                            let is_selected = selected().contains(pkg);
                                            let status = statuses().get(pkg).cloned().unwrap_or(PkgStatus::Idle);
                                            rsx! {
                                                tr { key: "{pkg_name}",
                                                    td {
                                                        input {
                                                            r#type: "checkbox",
                                                            checked: is_selected,
                                                            oninput: {
                                                                let pkg_name = pkg_name.clone();
                                                                move |_| toggle_select(pkg_name.clone())
                                                            },
                                                        }
                                                    }
                                                    td { class: "mono", "{pkg_name}" }
                                                    td {
                                                        match &status {
                                                            PkgStatus::Idle => rsx! { span { style: "color: var(--text-secondary);", "-" } },
                                                            PkgStatus::Updating => rsx! { span { style: "color: var(--accent);", "Updating..." } },
                                                            PkgStatus::Done(msg) => rsx! { span { style: "color: var(--success);", "{msg}" } },
                                                            PkgStatus::Error(msg) => rsx! { span { style: "color: var(--error);", "{msg}" } },
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        div { style: "margin-top: 16px; display: flex; gap: 12px;",
                            p { style: "color: var(--text-secondary); font-size: 14px; flex: 1;",
                                "{selected().len()} package(s) selected"
                            }
                        }
                    }
                }
            }
        }
    }
}
