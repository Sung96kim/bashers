use dioxus::prelude::*;

use crate::commands::watch::{compute_diff_lines, DiffLine, DiffSegment};
use crate::gui::server_fns::run_command;

#[component]
pub fn WatchPage() -> Element {
    let mut command_input = use_signal(|| "".to_string());
    let mut interval_secs = use_signal(|| 2u64);
    let mut diff_enabled = use_signal(|| true);
    let mut running = use_signal(|| false);
    let mut output_lines = use_signal(Vec::<DiffLine>::new);
    let mut previous_output = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);

    let mut do_run = move || {
        let cmd_str = command_input();
        if cmd_str.trim().is_empty() {
            return;
        }
        running.set(true);
        error.set(None);
        previous_output.set(None);
        output_lines.set(vec![]);

        let parts: Vec<String> = cmd_str.split_whitespace().map(String::from).collect();
        if parts.is_empty() {
            return;
        }

        let program = parts[0].clone();
        let args: Vec<String> = parts[1..].to_vec();

        spawn(async move {
            loop {
                if !running() {
                    break;
                }
                match run_command(program.clone(), args.clone()).await {
                    Ok(output) => {
                        let lines = if diff_enabled() {
                            if let Some(prev) = previous_output() {
                                compute_diff_lines(&prev, &output)
                            } else {
                                output.lines()
                                    .map(|l| DiffLine { segments: vec![DiffSegment::Same(l.to_string())] })
                                    .collect()
                            }
                        } else {
                            output.lines()
                                .map(|l| DiffLine { segments: vec![DiffSegment::Same(l.to_string())] })
                                .collect()
                        };
                        output_lines.set(lines);
                        previous_output.set(Some(output));
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(interval_secs())).await;
            }
        });
    };

    let do_stop = move |_| {
        running.set(false);
    };

    rsx! {
        div {
            h2 { "Watch" }
            div { class: "card", style: "margin-top: 16px;",
                div { style: "display: flex; gap: 12px; align-items: end;",
                    div { style: "flex: 1;",
                        label { style: "font-size: 12px; color: var(--text-secondary); display: block; margin-bottom: 4px;",
                            "Command"
                        }
                        input {
                            placeholder: "e.g. ls -la",
                            value: "{command_input}",
                            oninput: move |e| command_input.set(e.value()),
                            onkeypress: move |e: Event<KeyboardData>| {
                                if e.key() == Key::Enter && !running() {
                                    do_run();
                                }
                            },
                            disabled: running(),
                        }
                    }
                    div { style: "width: 100px;",
                        label { style: "font-size: 12px; color: var(--text-secondary); display: block; margin-bottom: 4px;",
                            "Interval (s)"
                        }
                        input {
                            r#type: "number",
                            value: "{interval_secs}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<u64>() {
                                    if v > 0 {
                                        interval_secs.set(v);
                                    }
                                }
                            },
                            disabled: running(),
                        }
                    }
                    div {
                        label { style: "font-size: 12px; color: var(--text-secondary); display: block; margin-bottom: 4px;",
                            "Diff"
                        }
                        input {
                            r#type: "checkbox",
                            checked: diff_enabled(),
                            oninput: move |e| diff_enabled.set(e.checked()),
                            disabled: running(),
                        }
                    }
                    if running() {
                        button { class: "btn btn-secondary", onclick: do_stop, "Stop" }
                    } else {
                        button { class: "btn", onclick: move |_| do_run(), "Run" }
                    }
                }
            }

            if let Some(err) = error() {
                div { class: "error-banner", style: "margin-top: 16px;",
                    "Error: {err}"
                }
            }

            div {
                class: "card mono",
                style: "margin-top: 16px; max-height: 500px; overflow-y: auto; white-space: pre-wrap;",
                for (i, line) in output_lines().iter().enumerate() {
                    div { key: "{i}",
                        for (j, segment) in line.segments.iter().enumerate() {
                            match segment {
                                DiffSegment::Same(text) => rsx! {
                                    span { key: "{j}", "{text}" }
                                },
                                DiffSegment::Added(text) => rsx! {
                                    span { key: "{j}", class: "diff-added", "{text}" }
                                },
                            }
                        }
                    }
                }
                if output_lines().is_empty() && !running() {
                    p { style: "color: var(--text-secondary); text-align: center; padding: 24px;",
                        "Enter a command and click Run to start watching"
                    }
                }
            }
        }
    }
}
