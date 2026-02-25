use dioxus::prelude::*;

use crate::commands::show::DependencyInfo;
use crate::gui::server_fns::list_dependencies;
use crate::utils::project::ProjectType;

#[component]
pub fn ShowPage() -> Element {
    let mut search = use_signal(String::new);
    let deps_resource = use_resource(|| async { list_dependencies().await });

    rsx! {
        div {
            match &*deps_resource.read() {
                None => rsx! {
                    div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 16px;",
                        h2 { "Packages" }
                    }
                    p { style: "color: var(--text-secondary);", "Loading packages..." }
                },
                Some(Err(err)) => rsx! {
                    div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 16px;",
                        h2 { "Packages" }
                    }
                    div { class: "error-banner",
                        "Error: {err}"
                    }
                },
                Some(Ok((project_type, deps))) => {
                    let search_val = search().to_lowercase();
                    let filtered: Vec<&DependencyInfo> = if search_val.is_empty() {
                        deps.iter().collect()
                    } else {
                        deps.iter()
                            .filter(|d| d.name.to_lowercase().contains(&search_val))
                            .collect()
                    };

                    rsx! {
                        div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 16px;",
                            h2 { "Packages" }
                            span {
                                class: match project_type {
                                    ProjectType::Uv => "badge badge-uv",
                                    ProjectType::Poetry => "badge badge-poetry",
                                    ProjectType::Cargo => "badge badge-cargo",
                                },
                                match project_type {
                                    ProjectType::Uv => "Uv",
                                    ProjectType::Poetry => "Poetry",
                                    ProjectType::Cargo => "Cargo",
                                }
                            }
                        }

                        div { style: "margin-bottom: 16px;",
                            input {
                                placeholder: "Filter packages...",
                                value: "{search}",
                                oninput: move |e| search.set(e.value()),
                            }
                        }

                        div { class: "card",
                            table {
                                thead {
                                    tr {
                                        th { "Package" }
                                        th { "Version" }
                                    }
                                }
                                tbody {
                                    for dep in filtered.iter() {
                                        tr { key: "{dep.name}",
                                            td { class: "mono", "{dep.name}" }
                                            td { class: "mono",
                                                {dep.version.as_deref().unwrap_or("-")}
                                            }
                                        }
                                    }
                                }
                            }
                            if filtered.is_empty() {
                                p { style: "padding: 16px; color: var(--text-secondary); text-align: center;",
                                    "No packages found"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
