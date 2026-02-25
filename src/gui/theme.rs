pub const BG_MAIN: &str = "#ffffff";
pub const BG_SIDEBAR: &str = "#f5f5f7";
pub const TEXT_PRIMARY: &str = "#1d1d1f";
pub const TEXT_SECONDARY: &str = "#6e6e73";
pub const ACCENT: &str = "#0071e3";
pub const SUCCESS: &str = "#34c759";
pub const ERROR: &str = "#ff3b30";
#[allow(dead_code)]
pub const WARNING: &str = "#ff9500";
pub const BORDER: &str = "#d2d2d7";

pub const FONT_STACK: &str = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";
pub const FONT_MONO: &str = "'SF Mono', 'Fira Code', 'Cascadia Code', monospace";

pub const SIDEBAR_WIDTH: &str = "220px";

pub fn global_css() -> String {
    format!(
        r#"
        :root {{
            --bg-main: {BG_MAIN};
            --bg-sidebar: {BG_SIDEBAR};
            --text-primary: {TEXT_PRIMARY};
            --text-secondary: {TEXT_SECONDARY};
            --accent: {ACCENT};
            --success: {SUCCESS};
            --error: {ERROR};
            --warning: {WARNING};
            --border: {BORDER};
            --hover-bg: rgba(0,0,0,0.04);
            --active-bg: rgba(0,113,227,0.06);
            --card-shadow: rgba(0,0,0,0.04);
            --scrollbar-thumb: rgba(0,0,0,0.15);
            --scrollbar-hover: rgba(0,0,0,0.25);
            --input-bg: {BG_MAIN};
            --error-banner-bg: #ffebee;
            --badge-uv-bg: #e8f5e9;
            --badge-uv-text: #2e7d32;
            --badge-poetry-bg: #e3f2fd;
            --badge-poetry-text: #1565c0;
            --badge-cargo-bg: #fff3e0;
            --badge-cargo-text: #e65100;
        }}
        .dark {{
            --bg-main: #1c1c1e;
            --bg-sidebar: #2c2c2e;
            --text-primary: #f5f5f7;
            --text-secondary: #8e8e93;
            --accent: #0a84ff;
            --success: #30d158;
            --error: #ff453a;
            --warning: #ff9f0a;
            --border: #3a3a3c;
            --hover-bg: rgba(255,255,255,0.06);
            --active-bg: rgba(10,132,255,0.15);
            --card-shadow: rgba(0,0,0,0.3);
            --scrollbar-thumb: rgba(255,255,255,0.2);
            --scrollbar-hover: rgba(255,255,255,0.3);
            --input-bg: #2c2c2e;
            --error-banner-bg: rgba(255,69,58,0.12);
            --badge-uv-bg: rgba(48,209,88,0.15);
            --badge-uv-text: #30d158;
            --badge-poetry-bg: rgba(10,132,255,0.15);
            --badge-poetry-text: #0a84ff;
            --badge-cargo-bg: rgba(255,159,10,0.15);
            --badge-cargo-text: #ff9f0a;
        }}
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: {FONT_STACK};
            color: var(--text-primary);
            background: var(--bg-main);
            -webkit-font-smoothing: antialiased;
        }}
        .sidebar {{
            width: {SIDEBAR_WIDTH};
            min-height: 100vh;
            background: var(--bg-sidebar);
            border-right: 1px solid var(--border);
            padding: 20px 0;
            position: fixed;
            left: 0;
            top: 0;
            display: flex;
            flex-direction: column;
        }}
        .main-content {{
            margin-left: {SIDEBAR_WIDTH};
            padding: 28px 32px;
            min-height: 100vh;
            background: var(--bg-main);
        }}
        .nav-item {{
            display: block;
            padding: 10px 20px;
            color: var(--text-secondary);
            cursor: pointer;
            text-decoration: none;
            font-size: 14px;
            border-left: 3px solid transparent;
            transition: all 0.15s ease;
        }}
        .nav-item:hover {{
            background: var(--hover-bg);
            color: var(--text-primary);
        }}
        .nav-item.active {{
            color: var(--accent);
            border-left-color: var(--accent);
            font-weight: 600;
            background: var(--active-bg);
        }}
        .card {{
            background: var(--bg-main);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 16px;
            margin-bottom: 16px;
            box-shadow: 0 1px 3px var(--card-shadow);
        }}
        .btn {{
            background: var(--accent);
            color: white;
            border: none;
            border-radius: 8px;
            padding: 8px 20px;
            font-size: 14px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.15s ease;
        }}
        .btn:hover {{
            opacity: 0.9;
            transform: translateY(-0.5px);
        }}
        .btn:active {{
            transform: translateY(0);
        }}
        .btn:disabled {{
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }}
        .btn-secondary {{
            background: transparent;
            color: var(--accent);
            border: 1px solid var(--accent);
        }}
        .btn-secondary:hover {{
            background: var(--active-bg);
            transform: translateY(-0.5px);
        }}
        input, .input {{
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 8px 12px;
            font-size: 14px;
            outline: none;
            width: 100%;
            background: var(--input-bg);
            color: var(--text-primary);
            transition: border-color 0.15s ease, box-shadow 0.15s ease;
        }}
        input:focus {{
            border-color: var(--accent);
            box-shadow: 0 0 0 3px rgba(0,113,227,0.12);
        }}
        .badge {{
            display: inline-block;
            padding: 2px 8px;
            border-radius: 6px;
            font-size: 12px;
            font-weight: 600;
        }}
        .badge-uv {{ background: var(--badge-uv-bg); color: var(--badge-uv-text); }}
        .badge-poetry {{ background: var(--badge-poetry-bg); color: var(--badge-poetry-text); }}
        .badge-cargo {{ background: var(--badge-cargo-bg); color: var(--badge-cargo-text); }}
        .error-banner {{
            background: var(--error-banner-bg);
            color: var(--error);
            border: 1px solid var(--error);
            border-radius: 10px;
            padding: 12px 16px;
            margin-bottom: 16px;
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 14px;
        }}
        .mono {{
            font-family: {FONT_MONO};
            font-size: 13px;
        }}
        .diff-added {{
            color: var(--success);
            font-weight: 600;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        th, td {{
            text-align: left;
            padding: 10px 12px;
            border-bottom: 1px solid var(--border);
        }}
        th {{
            font-weight: 600;
            color: var(--text-secondary);
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}
        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(4px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}
        .spinner {{
            display: inline-block;
            width: 14px;
            height: 14px;
            border: 2px solid rgba(255,255,255,0.3);
            border-top-color: white;
            border-radius: 50%;
            animation: spin 0.6s linear infinite;
            vertical-align: middle;
        }}
        .spinner-dark {{
            border-color: rgba(0,0,0,0.15);
            border-top-color: var(--text-secondary);
        }}
        .close-btn {{
            background: none;
            border: none;
            color: var(--text-secondary);
            cursor: pointer;
            font-size: 14px;
            padding: 4px 6px;
            border-radius: 6px;
            line-height: 1;
            transition: all 0.12s ease;
        }}
        .close-btn:hover {{
            background: rgba(255,59,48,0.1);
            color: var(--error);
        }}
        .log-count {{
            font-size: 10px;
            color: var(--text-secondary);
            margin-left: 4px;
        }}
        .splitter {{
            width: 6px;
            cursor: col-resize;
            background: var(--border);
            transition: background 0.15s ease;
            flex-shrink: 0;
            border-radius: 3px;
            margin: 0 2px;
        }}
        .splitter:hover {{
            background: var(--accent);
        }}
        .splitter-h {{
            width: 6px;
            cursor: col-resize;
            background: var(--border);
            transition: background 0.15s ease;
            flex-shrink: 0;
            margin: 0;
        }}
        .splitter-h:hover {{
            background: var(--accent);
        }}
        .log-header {{
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 6px 10px;
            background: #2c2c2e;
            border-radius: 12px 12px 0 0;
            border-bottom: 1px solid #3a3a3c;
        }}
        .log-header span {{
            color: #f5f5f7;
        }}
        .pinned-indicator {{
            display: inline-block;
            width: 6px;
            height: 6px;
            border-radius: 50%;
            background: var(--accent);
            flex-shrink: 0;
        }}
        h2 {{
            font-size: 22px;
            font-weight: 700;
            letter-spacing: -0.3px;
        }}
        h3 {{
            font-size: 14px;
            font-weight: 600;
        }}
        .theme-toggle {{
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 10px 20px;
            cursor: pointer;
            color: var(--text-secondary);
            font-size: 13px;
            transition: color 0.15s ease;
            border: none;
            background: none;
            width: 100%;
            text-align: left;
        }}
        .theme-toggle:hover {{
            color: var(--text-primary);
        }}
        ::-webkit-scrollbar {{
            width: 6px;
            height: 6px;
        }}
        ::-webkit-scrollbar-track {{
            background: transparent;
        }}
        ::-webkit-scrollbar-thumb {{
            background: var(--scrollbar-thumb);
            border-radius: 3px;
        }}
        ::-webkit-scrollbar-thumb:hover {{
            background: var(--scrollbar-hover);
        }}
        "#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_css_contains_key_styles() {
        let css = global_css();
        assert!(css.contains(BG_MAIN));
        assert!(css.contains(BG_SIDEBAR));
        assert!(css.contains(ACCENT));
        assert!(css.contains(FONT_STACK));
    }

    #[test]
    fn test_global_css_contains_spinner() {
        let css = global_css();
        assert!(css.contains("@keyframes spin"));
        assert!(css.contains(".spinner"));
    }

    #[test]
    fn test_global_css_contains_close_btn() {
        let css = global_css();
        assert!(css.contains(".close-btn"));
    }

    #[test]
    fn test_global_css_contains_splitter() {
        let css = global_css();
        assert!(css.contains(".splitter"));
        assert!(css.contains(".splitter-h"));
        assert!(css.contains("col-resize"));
    }

    #[test]
    fn test_global_css_contains_log_header() {
        let css = global_css();
        assert!(css.contains(".log-header"));
    }

    #[test]
    fn test_global_css_contains_pinned_indicator() {
        let css = global_css();
        assert!(css.contains(".pinned-indicator"));
    }

    #[test]
    fn test_global_css_contains_dark_mode() {
        let css = global_css();
        assert!(css.contains(".dark"));
        assert!(css.contains("--bg-main"));
        assert!(css.contains("--text-primary"));
        assert!(css.contains("--accent"));
    }

    #[test]
    fn test_global_css_contains_theme_toggle() {
        let css = global_css();
        assert!(css.contains(".theme-toggle"));
    }

    #[test]
    fn test_constants_are_valid_hex_colors() {
        let colors = [
            BG_MAIN,
            BG_SIDEBAR,
            TEXT_PRIMARY,
            TEXT_SECONDARY,
            ACCENT,
            SUCCESS,
            ERROR,
            WARNING,
            BORDER,
        ];
        for c in colors {
            assert!(c.starts_with('#'), "Color {c} should start with #");
            assert!(c.len() == 7, "Color {c} should be 7 chars (#rrggbb)");
        }
    }
}
