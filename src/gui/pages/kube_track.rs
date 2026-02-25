use std::time::Duration;

use dioxus::prelude::*;

use crate::commands::kube::track::PodInfo;
use crate::gui::server_fns::{get_pod_logs, search_pods};

fn pop_out_logs(pod_name: &str, logs: &[String]) {
    let html_lines: Vec<String> = logs.iter().map(|l| ansi_to_html(l)).collect();
    let body = html_lines.join("\n");
    let full_html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>{pod_name}</title><style>body{{background:#1d1d1f;color:#f5f5f7;font-family:'SF Mono','Fira Code','Cascadia Code',monospace;font-size:13px;padding:12px;white-space:pre-wrap;margin:0;}}</style></head><body>{body}</body></html>"#,
    );
    let escaped = full_html
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n");
    let js = format!(
        "var b=new Blob(['{}'],{{type:'text/html'}});window.open(URL.createObjectURL(b),'_blank');",
        escaped,
    );
    document::eval(&js);
}

const MAX_LINES_PER_POD: usize = 5000;
const POLL_INTERVAL_MS: u64 = 2000;

#[derive(Clone, Debug)]
struct PodLog {
    pod: PodInfo,
    lines: Vec<String>,
    active: bool,
}

pub fn ansi_to_html(line: &str) -> String {
    let escaped = line
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let mut result = String::with_capacity(escaped.len() + 64);
    let mut open_spans: usize = 0;
    let bytes = escaped.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            let start = i + 2;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'm' {
                if !bytes[end].is_ascii_digit() && bytes[end] != b';' {
                    break;
                }
                end += 1;
            }
            if end < bytes.len() && bytes[end] == b'm' {
                let codes_str = &escaped[start..end];
                let codes: Vec<u8> = codes_str
                    .split(';')
                    .filter_map(|s| s.parse().ok())
                    .collect();

                for _ in 0..open_spans {
                    result.push_str("</span>");
                }
                open_spans = 0;

                if codes.is_empty() || codes == [0] {
                    i = end + 1;
                    continue;
                }

                let mut styles = Vec::new();
                for &code in &codes {
                    match code {
                        1 => styles.push("font-weight:bold"),
                        2 => styles.push("opacity:0.7"),
                        3 => styles.push("font-style:italic"),
                        4 => styles.push("text-decoration:underline"),
                        30 => styles.push("color:#1d1d1f"),
                        31 => styles.push("color:#ff3b30"),
                        32 => styles.push("color:#34c759"),
                        33 => styles.push("color:#ff9500"),
                        34 => styles.push("color:#007aff"),
                        35 => styles.push("color:#af52de"),
                        36 => styles.push("color:#5ac8fa"),
                        37 => styles.push("color:#8e8e93"),
                        90 => styles.push("color:#8e8e93"),
                        91 => styles.push("color:#ff6961"),
                        92 => styles.push("color:#77dd77"),
                        93 => styles.push("color:#fdfd96"),
                        94 => styles.push("color:#89cff0"),
                        95 => styles.push("color:#c3b1e1"),
                        96 => styles.push("color:#99e5ff"),
                        97 => styles.push("color:#f5f5f7"),
                        _ => {}
                    }
                }

                if !styles.is_empty() {
                    result.push_str(&format!("<span style=\"{}\">", styles.join(";")));
                    open_spans = 1;
                }

                i = end + 1;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    for _ in 0..open_spans {
        result.push_str("</span>");
    }

    result
}

#[component]
pub fn KubeTrackPage() -> Element {
    let mut pattern_input = use_signal(String::new);
    let mut pods = use_signal(Vec::<PodLog>::new);
    let mut pinned_pods = use_signal(Vec::<String>::new);
    let mut error = use_signal(|| None::<String>);
    let mut searching = use_signal(|| false);
    let mut streaming = use_signal(|| false);
    let mut sidebar_width = use_signal(|| 250.0f64);
    let mut dragging = use_signal(|| false);

    let do_stop = move |_| {
        streaming.set(false);
        let mut current = pods();
        for pod_log in current.iter_mut() {
            pod_log.active = false;
        }
        pods.set(current);
    };

    let mut close_pod = move |key: String| {
        let mut current = pods();
        if let Some(pod_log) = current.iter_mut().find(|p| p.pod.key() == key) {
            pod_log.active = false;
        }
        current.retain(|p| p.pod.key() != key);
        let mut current_pinned = pinned_pods();
        current_pinned.retain(|k| k != &key);
        if current_pinned.is_empty() {
            if let Some(first) = current.first() {
                current_pinned.push(first.pod.key());
            }
        }
        pinned_pods.set(current_pinned);
        if current.is_empty() {
            streaming.set(false);
        }
        pods.set(current);
    };

    let mut toggle_pin = move |key: String| {
        let mut current = pinned_pods();
        if current.contains(&key) {
            if current.len() > 1 {
                current.retain(|k| k != &key);
            }
        } else {
            current.push(key);
        }
        pinned_pods.set(current);
    };

    let mut do_search = move || {
        let pattern = pattern_input();
        if pattern.trim().is_empty() {
            return;
        }

        streaming.set(false);
        let mut old_pods = pods();
        for pod_log in old_pods.iter_mut() {
            pod_log.active = false;
        }
        pods.set(vec![]);

        searching.set(true);
        error.set(None);
        pinned_pods.set(vec![]);

        let patterns: Vec<String> = pattern.split_whitespace().map(String::from).collect();

        spawn(async move {
            let result = search_pods(patterns).await;

            searching.set(false);

            let matched = match result {
                Ok(m) => m,
                Err(e) => {
                    error.set(Some(e.to_string()));
                    return;
                }
            };

            if matched.is_empty() {
                error.set(Some(format!("No pods found matching: {pattern}")));
                return;
            }

            let pod_logs: Vec<PodLog> = matched
                .into_iter()
                .map(|p| PodLog {
                    pod: p,
                    lines: vec![],
                    active: true,
                })
                .collect();

            let first_key = pod_logs.first().map(|p| p.pod.key());
            if let Some(key) = first_key {
                pinned_pods.set(vec![key]);
            }
            pods.set(pod_logs);
            streaming.set(true);

            loop {
                tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;

                if !streaming() {
                    break;
                }

                let current = pods();
                let active_pods: Vec<(String, String, usize)> = current
                    .iter()
                    .filter(|p| p.active)
                    .map(|p| (p.pod.namespace.clone(), p.pod.name.clone(), p.lines.len()))
                    .collect();

                if active_pods.is_empty() {
                    break;
                }

                for (ns, name, existing_count) in active_pods {
                    let tail = if existing_count == 0 { 1000 } else { 100 };
                    let pod_key = format!("{ns}/{name}");

                    match get_pod_logs(ns, name, tail as u64).await {
                        Ok(new_lines) => {
                            if new_lines.is_empty() {
                                continue;
                            }
                            let mut current = pods();
                            if let Some(pod_log) =
                                current.iter_mut().find(|p| p.pod.key() == pod_key)
                            {
                                if existing_count == 0 {
                                    pod_log.lines = new_lines;
                                } else {
                                    pod_log.lines.extend(new_lines);
                                }
                                if pod_log.lines.len() > MAX_LINES_PER_POD {
                                    let drain_count = pod_log.lines.len() - MAX_LINES_PER_POD;
                                    pod_log.lines.drain(..drain_count);
                                }
                            }
                            pods.set(current);
                        }
                        Err(_) => {}
                    }
                }
            }
        });
    };

    let current_pods = pods();
    let current_pinned = pinned_pods();
    let sw = sidebar_width();
    let is_dragging = dragging();

    rsx! {
        div {
            style: if is_dragging { "user-select: none; cursor: col-resize;" } else { "" },
            onmousemove: move |e: Event<MouseData>| {
                if dragging() {
                    let x = e.page_coordinates().x;
                    let clamped = x.max(150.0).min(500.0);
                    sidebar_width.set(clamped);
                }
            },
            onmouseup: move |_| {
                dragging.set(false);
            },

            div { style: "display: flex; align-items: center; gap: 12px; margin-bottom: 0;",
                h2 { "Kube Track" }
                if streaming() {
                    span { style: "font-size: 12px; color: var(--success); font-weight: 600;",
                        "Streaming"
                    }
                }
            }

            div { class: "card", style: "margin-top: 16px;",
                div { style: "display: flex; gap: 12px; align-items: end;",
                    div { style: "flex: 1;",
                        label { style: "font-size: 12px; color: var(--text-secondary); display: block; margin-bottom: 4px;",
                            "Pod Pattern(s)"
                        }
                        input {
                            placeholder: "e.g. api-server worker-.*",
                            value: "{pattern_input}",
                            oninput: move |e| pattern_input.set(e.value()),
                            onkeypress: move |e: Event<KeyboardData>| {
                                if e.key() == Key::Enter {
                                    do_search();
                                }
                            },
                        }
                    }
                    if streaming() {
                        button {
                            class: "btn btn-secondary",
                            onclick: do_stop,
                            "Stop"
                        }
                    }
                    button {
                        class: "btn",
                        onclick: move |_| do_search(),
                        disabled: searching(),
                        if searching() {
                            span { class: "spinner" }
                        } else {
                            "Find Pods"
                        }
                    }
                }
            }

            if let Some(err) = error() {
                div { class: "error-banner", style: "margin-top: 16px;",
                    "Error: {err}"
                }
            }

            if !current_pods.is_empty() {
                div { style: "display: flex; margin-top: 16px; height: calc(100vh - 200px);",
                    div {
                        class: "card",
                        style: "width: {sw}px; min-width: 150px; overflow-y: auto; flex-shrink: 0;",
                        h3 { style: "font-size: 14px; color: var(--text-secondary); margin-bottom: 8px;",
                            "Pods ({current_pods.len()})"
                        }
                        for pod_log in current_pods.iter() {
                            {
                                let key = pod_log.pod.key();
                                let line_count = pod_log.lines.len();
                                let is_pinned = current_pinned.contains(&key);
                                let style = if is_pinned {
                                    "padding: 8px; cursor: pointer; background: var(--active-bg); border-radius: 4px; margin-bottom: 4px; display: flex; align-items: start; justify-content: space-between;"
                                } else {
                                    "padding: 8px; cursor: pointer; border-radius: 4px; margin-bottom: 4px; display: flex; align-items: start; justify-content: space-between;"
                                };
                                rsx! {
                                    div {
                                        key: "{key}",
                                        style: "{style}",
                                        div {
                                            style: "flex: 1; min-width: 0;",
                                            onclick: {
                                                let key = key.clone();
                                                move |_| toggle_pin(key.clone())
                                            },
                                            div { style: "display: flex; align-items: center; gap: 4px;",
                                                if is_pinned {
                                                    span { class: "pinned-indicator" }
                                                }
                                                span { class: "mono", style: "font-size: 13px; overflow: hidden; text-overflow: ellipsis;",
                                                    "{pod_log.pod.name}"
                                                }
                                            }
                                            div { style: "font-size: 11px; color: var(--text-secondary);",
                                                "{pod_log.pod.namespace}"
                                                if line_count > 0 {
                                                    span { class: "log-count", "{line_count}" }
                                                }
                                            }
                                        }
                                        button {
                                            class: "close-btn",
                                            onclick: {
                                                let key = key.clone();
                                                move |evt: Event<MouseData>| {
                                                    evt.stop_propagation();
                                                    close_pod(key.clone());
                                                }
                                            },
                                            "x"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div {
                        class: "splitter",
                        onmousedown: move |e: Event<MouseData>| {
                            e.prevent_default();
                            dragging.set(true);
                        },
                    }

                    div { style: "flex: 1; display: flex; gap: 0; min-width: 0; overflow: hidden;",
                        if current_pinned.is_empty() {
                            div {
                                class: "card mono",
                                style: "flex: 1; overflow-y: auto; white-space: pre-wrap; font-size: 13px;",
                                p { style: "color: var(--text-secondary); text-align: center; padding: 24px;",
                                    "Click a pod to pin it"
                                }
                            }
                        } else {
                            for (idx, pinned_key) in current_pinned.iter().enumerate() {
                                {
                                    let logs: Vec<String> = current_pods
                                        .iter()
                                        .find(|p| &p.pod.key() == pinned_key)
                                        .map(|p| p.lines.clone())
                                        .unwrap_or_default();
                                    let pod_name = current_pods
                                        .iter()
                                        .find(|p| &p.pod.key() == pinned_key)
                                        .map(|p| p.pod.name.clone())
                                        .unwrap_or_default();

                                    rsx! {
                                        if idx > 0 {
                                            div { class: "splitter-h" }
                                        }
                                        div {
                                            key: "{pinned_key}",
                                            style: "flex: 1; display: flex; flex-direction: column; min-width: 0; overflow: hidden;",
                                            div { class: "log-header",
                                                span { class: "mono", style: "font-size: 12px; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                                    "{pod_name}"
                                                }
                                                div { style: "display: flex; gap: 4px;",
                                                    button {
                                                        class: "close-btn",
                                                        title: "Pop out",
                                                        onclick: {
                                                            let logs_clone = logs.clone();
                                                            let name_clone = pod_name.clone();
                                                            move |_| {
                                                                pop_out_logs(&name_clone, &logs_clone);
                                                            }
                                                        },
                                                        "^"
                                                    }
                                                    if current_pinned.len() > 1 {
                                                        button {
                                                            class: "close-btn",
                                                            title: "Unpin",
                                                            onclick: {
                                                                let pk = pinned_key.clone();
                                                                move |_| toggle_pin(pk.clone())
                                                            },
                                                            "x"
                                                        }
                                                    }
                                                }
                                            }
                                            div {
                                                class: "mono",
                                                style: "flex: 1; overflow-y: auto; white-space: pre-wrap; font-size: 13px; padding: 8px; background: #1d1d1f; color: #f5f5f7; border-radius: 0 0 8px 8px;",
                                                if logs.is_empty() {
                                                    if streaming() {
                                                        p { style: "color: var(--text-secondary); text-align: center; padding: 24px;",
                                                            "Waiting for logs..."
                                                        }
                                                    }
                                                } else {
                                                    for (i, line) in logs.iter().enumerate() {
                                                        {
                                                            let html = ansi_to_html(line);
                                                            rsx! {
                                                                div {
                                                                    key: "{i}",
                                                                    dangerous_inner_html: "{html}",
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_to_html_plain_text() {
        assert_eq!(ansi_to_html("hello world"), "hello world");
    }

    #[test]
    fn test_ansi_to_html_escapes_html() {
        assert_eq!(ansi_to_html("<b>test</b>"), "&lt;b&gt;test&lt;/b&gt;");
        assert_eq!(ansi_to_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_ansi_to_html_red_text() {
        let input = "\x1b[31mERROR\x1b[0m ok";
        let output = ansi_to_html(input);
        assert!(output.contains("color:#ff3b30"));
        assert!(output.contains("ERROR"));
        assert!(output.contains("ok"));
    }

    #[test]
    fn test_ansi_to_html_bold() {
        let input = "\x1b[1mbold\x1b[0m";
        let output = ansi_to_html(input);
        assert!(output.contains("font-weight:bold"));
        assert!(output.contains("bold"));
    }

    #[test]
    fn test_ansi_to_html_green_text() {
        let input = "\x1b[32mSUCCESS\x1b[0m";
        let output = ansi_to_html(input);
        assert!(output.contains("color:#34c759"));
    }

    #[test]
    fn test_ansi_to_html_combined_codes() {
        let input = "\x1b[1;31mBOLD RED\x1b[0m";
        let output = ansi_to_html(input);
        assert!(output.contains("font-weight:bold"));
        assert!(output.contains("color:#ff3b30"));
    }

    #[test]
    fn test_ansi_to_html_bright_colors() {
        let input = "\x1b[91mbright red\x1b[0m";
        let output = ansi_to_html(input);
        assert!(output.contains("color:#ff6961"));
    }

    #[test]
    fn test_ansi_to_html_no_unclosed_spans() {
        let input = "\x1b[31mno reset";
        let output = ansi_to_html(input);
        let opens = output.matches("<span").count();
        let closes = output.matches("</span>").count();
        assert_eq!(opens, closes);
    }

    #[test]
    fn test_ansi_to_html_multiple_sequences() {
        let input = "\x1b[31mred\x1b[32mgreen\x1b[0mnormal";
        let output = ansi_to_html(input);
        assert!(output.contains("red"));
        assert!(output.contains("green"));
        assert!(output.contains("normal"));
        let opens = output.matches("<span").count();
        let closes = output.matches("</span>").count();
        assert_eq!(opens, closes);
    }

    #[test]
    fn test_ansi_to_html_empty_string() {
        assert_eq!(ansi_to_html(""), "");
    }
}
