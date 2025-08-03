
use yew::prelude::*;
use web_sys::{HtmlTextAreaElement, HtmlSelectElement, HtmlInputElement};
use regex::Regex;
use serde::{Deserialize, Serialize};
use gloo_console::log;
use gloo_timers::callback::Timeout;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogEntry {
    timestamp: Option<String>,
    level: String,
    message: String,
    source: Option<String>,
}

#[derive(Debug, Clone)]
struct LogStats {
    total_lines: usize,
    error_count: usize,
    warn_count: usize,
    info_count: usize,
    debug_count: usize,
    trace_count: usize,
    unique_sources: HashMap<String, usize>,
}

impl Default for LogStats {
    fn default() -> Self {
        Self {
            total_lines: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            debug_count: 0,
            trace_count: 0,
            unique_sources: HashMap::new(),
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let log_input = use_state(|| String::new());
    let parsed_logs = use_state(|| Vec::<LogEntry>::new());
    let stats = use_state(|| LogStats::default());
    let filter_level = use_state(|| "all".to_string());
    let search_term = use_state(|| String::new());
    let is_parsing = use_state(|| false);

    let oninput = {
        let log_input = log_input.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlTextAreaElement>() {
                log_input.set(input.value());
            }
        })
    };

    let on_search_input = {
        let search_term = search_term.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                search_term.set(input.value());
            }
        })
    };

    let on_filter_change = {
        let filter_level = filter_level.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                filter_level.set(select.value());
            }
        })
    };

    let parse_logs = {
        let log_input = log_input.clone();
        let parsed_logs = parsed_logs.clone();
        let stats = stats.clone();
        let is_parsing = is_parsing.clone();
        
        Callback::from(move |_: MouseEvent| {
            let log_input = log_input.clone();
            let parsed_logs = parsed_logs.clone();
            let stats = stats.clone();
            let is_parsing = is_parsing.clone();
            
            is_parsing.set(true);
            
            // Parse logs in a timeout to prevent blocking UI
            let timeout_closure = Timeout::new(10, move || {
                let entries = parse_log_entries(&log_input);
                let log_stats = calculate_stats(&entries);
                
                parsed_logs.set(entries);
                stats.set(log_stats);
                is_parsing.set(false);
            });
            timeout_closure.forget();
        })
    };

    let clear_logs = {
        let log_input = log_input.clone();
        let parsed_logs = parsed_logs.clone();
        let stats = stats.clone();
        
        Callback::from(move |_: MouseEvent| {
            log_input.set(String::new());
            parsed_logs.set(Vec::new());
            stats.set(LogStats::default());
        })
    };

    // Filter logs based on level and search term
    let filtered_logs: Vec<LogEntry> = parsed_logs
        .iter()
        .filter(|entry| {
            let level_match = match filter_level.as_str() {
                "all" => true,
                level => entry.level.to_lowercase() == level,
            };
            
            let search_match = if search_term.is_empty() {
                true
            } else {
                entry.message.to_lowercase().contains(&search_term.to_lowercase()) ||
                entry.source.as_ref().map_or(false, |s| s.to_lowercase().contains(&search_term.to_lowercase()))
            };
            
            level_match && search_match
        })
        .cloned()
        .collect();

    html! {
        <div class="app-container">
            <style>
                {
                    "
                    .app-container {
                        max-width: 1200px;
                        margin: 0 auto;
                        padding: 20px;
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    }
                    .header {
                        text-align: center;
                        margin-bottom: 30px;
                        color: #333;
                    }
                    .input-section {
                        margin-bottom: 20px;
                    }
                    .log-textarea {
                        width: 100%;
                        height: 200px;
                        padding: 10px;
                        border: 2px solid #ddd;
                        border-radius: 8px;
                        font-family: 'Courier New', monospace;
                        font-size: 12px;
                        resize: vertical;
                    }
                    .controls {
                        display: flex;
                        gap: 10px;
                        margin: 10px 0;
                        flex-wrap: wrap;
                        align-items: center;
                    }
                    .btn {
                        padding: 10px 20px;
                        border: none;
                        border-radius: 5px;
                        cursor: pointer;
                        font-weight: bold;
                        transition: background-color 0.3s;
                    }
                    .btn-primary {
                        background-color: #007bff;
                        color: white;
                    }
                    .btn-primary:hover {
                        background-color: #0056b3;
                    }
                    .btn-secondary {
                        background-color: #6c757d;
                        color: white;
                    }
                    .btn-secondary:hover {
                        background-color: #545b62;
                    }
                    .btn:disabled {
                        opacity: 0.6;
                        cursor: not-allowed;
                    }
                    .stats-grid {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
                        gap: 15px;
                        margin: 20px 0;
                    }
                    .stat-card {
                        background: #f8f9fa;
                        padding: 15px;
                        border-radius: 8px;
                        text-align: center;
                        border: 1px solid #e9ecef;
                    }
                    .stat-number {
                        font-size: 24px;
                        font-weight: bold;
                        color: #495057;
                    }
                    .stat-label {
                        font-size: 12px;
                        color: #6c757d;
                        text-transform: uppercase;
                    }
                    .filters {
                        display: flex;
                        gap: 10px;
                        margin: 20px 0;
                        flex-wrap: wrap;
                        align-items: center;
                    }
                    .form-control {
                        padding: 8px 12px;
                        border: 1px solid #ddd;
                        border-radius: 4px;
                    }
                    .log-entries {
                        max-height: 400px;
                        overflow-y: auto;
                        border: 1px solid #ddd;
                        border-radius: 8px;
                    }
                    .log-entry {
                        padding: 10px;
                        border-bottom: 1px solid #eee;
                        font-family: 'Courier New', monospace;
                        font-size: 12px;
                    }
                    .log-entry:last-child {
                        border-bottom: none;
                    }
                    .log-level {
                        display: inline-block;
                        padding: 2px 6px;
                        border-radius: 3px;
                        font-weight: bold;
                        font-size: 10px;
                        margin-right: 8px;
                        min-width: 50px;
                        text-align: center;
                    }
                    .level-error { background-color: #dc3545; color: white; }
                    .level-warn { background-color: #ffc107; color: black; }
                    .level-info { background-color: #17a2b8; color: white; }
                    .level-debug { background-color: #6c757d; color: white; }
                    .level-trace { background-color: #343a40; color: white; }
                    .level-unknown { background-color: #e9ecef; color: black; }
                    .log-timestamp {
                        color: #6c757d;
                        margin-right: 8px;
                    }
                    .log-source {
                        color: #007bff;
                        margin-right: 8px;
                    }
                    .no-logs {
                        text-align: center;
                        padding: 40px;
                        color: #6c757d;
                    }
                    .loading {
                        text-align: center;
                        padding: 20px;
                        color: #007bff;
                    }
                    "
                }
            </style>
            
            <div class="header">
                <h1>{"🔍 Warp Log Inspector"}</h1>
                <p>{"Analyze and filter your application logs with ease"}</p>
            </div>

            <div class="input-section">
                <textarea 
                    class="log-textarea"
                    placeholder="Paste your log entries here...\n\nSupported formats:\n- Standard format: [TIMESTAMP] LEVEL MESSAGE\n- JSON logs\n- Custom formats with timestamps, levels, and messages"
                    value={(*log_input).clone()}
                    oninput={oninput}
                />
                
                <div class="controls">
                    <button 
                        class="btn btn-primary" 
                        onclick={parse_logs}
                        disabled={log_input.is_empty() || *is_parsing}
                    >
                        {if *is_parsing { "⏳ Parsing..." } else { "🔍 Parse Logs" }}
                    </button>
                    
                    <button 
                        class="btn btn-secondary" 
                        onclick={clear_logs}
                    >
                        {"🗑️ Clear"}
                    </button>
                    
                    <span style="margin-left: auto; color: #6c757d; font-size: 12px;">
                        {format!("{} characters", log_input.len())}
                    </span>
                </div>
            </div>

            if !parsed_logs.is_empty() {
                <div class="stats-section">
                    <h3>{"📊 Log Statistics"}</h3>
                    <div class="stats-grid">
                        <div class="stat-card">
                            <div class="stat-number">{stats.total_lines}</div>
                            <div class="stat-label">{"Total Lines"}</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-number">{stats.error_count}</div>
                            <div class="stat-label">{"Errors"}</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-number">{stats.warn_count}</div>
                            <div class="stat-label">{"Warnings"}</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-number">{stats.info_count}</div>
                            <div class="stat-label">{"Info"}</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-number">{stats.debug_count}</div>
                            <div class="stat-label">{"Debug"}</div>
                        </div>
                        <div class="stat-card">
                            <div class="stat-number">{stats.unique_sources.len()}</div>
                            <div class="stat-label">{"Sources"}</div>
                        </div>
                    </div>
                </div>

                <div class="filters">
                    <label>{"Filter by level: "}</label>
                    <select class="form-control" onchange={on_filter_change} value={(*filter_level).clone()}>
                        <option value="all">{"All Levels"}</option>
                        <option value="error">{"Error"}</option>
                        <option value="warn">{"Warning"}</option>
                        <option value="info">{"Info"}</option>
                        <option value="debug">{"Debug"}</option>
                        <option value="trace">{"Trace"}</option>
                    </select>
                    
                    <label style="margin-left: 20px;">{"Search: "}</label>
                    <input 
                        type="text" 
                        class="form-control" 
                        placeholder="Search messages..."
                        value={(*search_term).clone()}
                        oninput={on_search_input}
                    />
                    
                    <span style="margin-left: auto; color: #6c757d; font-size: 12px;">
                        {format!("{} / {} entries", filtered_logs.len(), parsed_logs.len())}
                    </span>
                </div>

                <div class="log-entries">
                    if filtered_logs.is_empty() {
                        <div class="no-logs">
                            {"No log entries match your current filters"}
                        </div>
                    } else {
                        { for filtered_logs.iter().map(|entry| render_log_entry(entry)) }
                    }
                </div>
            } else if *is_parsing {
                <div class="loading">
                    {"⏳ Parsing logs..."}
                </div>
            }
        </div>
    }
}

fn render_log_entry(entry: &LogEntry) -> Html {
    let level_class = format!("log-level level-{}", entry.level.to_lowercase());
    
    html! {
        <div class="log-entry">
            <span class={level_class}>{entry.level.to_uppercase()}</span>
            if let Some(timestamp) = &entry.timestamp {
                <span class="log-timestamp">{timestamp}</span>
            }
            if let Some(source) = &entry.source {
                <span class="log-source">{format!("[{}]:", source)}</span>
            }
            <span>{&entry.message}</span>
        </div>
    }
}

fn parse_log_entries(input: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    
    // Common log patterns
    let patterns = vec![
        // Standard format: [2023-01-01T12:00:00Z] ERROR This is an error message
        Regex::new(r"^\[(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[^\]]*?)\]\s+(ERROR|WARN|INFO|DEBUG|TRACE)\s+(.+)$").unwrap(),
        // Without brackets: 2023-01-01T12:00:00Z ERROR This is an error message
        Regex::new(r"^(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[^\s]*?)\s+(ERROR|WARN|INFO|DEBUG|TRACE)\s+(.+)$").unwrap(),
        // Level first: ERROR [2023-01-01T12:00:00Z] This is an error message
        Regex::new(r"^(ERROR|WARN|INFO|DEBUG|TRACE)\s+\[(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[^\]]*?)\]\s+(.+)$").unwrap(),
        // Just level and message: ERROR This is an error message
        Regex::new(r"^(ERROR|WARN|INFO|DEBUG|TRACE)\s+(.+)$").unwrap(),
    ];
    
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let mut parsed = false;
        
        // Try each pattern
        for pattern in &patterns {
            if let Some(captures) = pattern.captures(line) {
                let entry = match captures.len() {
                    4 => LogEntry {
                        timestamp: Some(captures[1].to_string()),
                        level: captures[2].to_string(),
                        message: captures[3].to_string(),
                        source: None,
                    },
                    3 if pattern.as_str().contains("ERROR|WARN|INFO|DEBUG|TRACE.*\\[") => LogEntry {
                        timestamp: Some(captures[2].to_string()),
                        level: captures[1].to_string(),
                        message: captures[3].to_string(),
                        source: None,
                    },
                    3 => LogEntry {
                        timestamp: None,
                        level: captures[1].to_string(),
                        message: captures[2].to_string(),
                        source: None,
                    },
                    _ => continue,
                };
                
                entries.push(entry);
                parsed = true;
                break;
            }
        }
        
        // If no pattern matched, treat as unknown level
        if !parsed {
            entries.push(LogEntry {
                timestamp: None,
                level: "unknown".to_string(),
                message: line.to_string(),
                source: None,
            });
        }
    }
    
    log!(format!("Parsed {} log entries", entries.len()));
    entries
}

fn calculate_stats(entries: &[LogEntry]) -> LogStats {
    let mut stats = LogStats::default();
    stats.total_lines = entries.len();
    
    for entry in entries {
        match entry.level.to_lowercase().as_str() {
            "error" => stats.error_count += 1,
            "warn" | "warning" => stats.warn_count += 1,
            "info" => stats.info_count += 1,
            "debug" => stats.debug_count += 1,
            "trace" => stats.trace_count += 1,
            _ => {},
        }
        
        if let Some(source) = &entry.source {
            *stats.unique_sources.entry(source.clone()).or_insert(0) += 1;
        }
    }
    
    stats
}

fn main() {
    yew::Renderer::<App>::new().render();
}



