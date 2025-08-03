
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
struct HttpTransaction {
    timestamp: String,
    method: Option<String>,
    url: Option<String>,
    status: Option<u16>,
    headers: Vec<(String, String)>,
    body: Option<String>,
    direction: String,
}

fn main() -> io::Result<()> {
    let path = "warp_network.log";
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut transactions: Vec<HttpTransaction> = vec![];
    let mut current: Option<HttpTransaction> = None;

    for line in reader.lines() {
        let line = line?;
        if line.contains("Request {") || line.contains("Response {") {
            if let Some(tx) = current.take() {
                transactions.push(tx);
            }

            let (timestamp, direction) = line.split_once("]: ").unwrap_or(("", ""));
            current = Some(HttpTransaction {
                timestamp: timestamp.trim_start_matches('[').to_string(),
                direction: if direction.contains("Request") { "Request" } else { "Response" }.to_string(),
                method: None,
                url: None,
                status: None,
                headers: vec![],
                body: None,
            });
        }

        if let Some(tx) = current.as_mut() {
            if line.contains("method:") {
                if let Some(method) = line.split("method: ").nth(1) {
                    tx.method = Some(method.trim().to_string());
                }
            }

            if line.contains("url:") && !line.contains("Response") {
                if let Some(url_part) = line.split("url: Url {").nth(1) {
                    let url = url_part
                        .split(',')
                        .find(|p| p.trim().starts_with("scheme"))
                        .and_then(|_| {
                            let scheme = line.split("scheme: ").nth(1)?.split(',').next()?.trim_matches('"');
                            let host = line.split("host: Some(Domain("").nth(1)?.split('"').next()?;
                            let path = line.split("path: "").nth(1)?.split('"').next()?;
                            Some(format!("{}://{}{}", scheme, host, path))
                        });
                    tx.url = url;
                }
            }

            if line.contains("url: ") && line.contains("Response") {
                if let Some(url) = line.split("url: ").nth(1) {
                    tx.url = Some(url.split(',').next()?.trim_matches('"').to_string());
                }
            }

            if line.contains("status:") {
                if let Some(status_str) = line.split("status: ").nth(1) {
                    tx.status = status_str.split(',').next()?.parse().ok();
                }
            }

            if line.contains("headers:") {
                if let Some(hdr_str) = line.split("headers: ").nth(1) {
                    let cleaned = hdr_str.trim_matches(|c| c == '{' || c == '}' || c == '"');
                    let headers = cleaned.split(", ")
                        .filter_map(|kv| kv.split_once(':'))
                        .map(|(k, v)| (k.trim().to_string(), v.trim_matches('"').to_string()))
                        .collect::<Vec<_>>();
                    tx.headers.extend(headers);
                }
            }

            if line.contains("Body {") {
                let body = line.split("Body {").nth(1)
                    .map(|b| b.trim().to_string());
                tx.body = body;
            }
        }
    }

    if let Some(tx) = current {
        transactions.push(tx);
    }

    for tx in transactions {
        println!("
==== {} [{}] ====", tx.direction, tx.timestamp);
        if let Some(method) = tx.method.as_ref() {
            println!("{} {}", method, tx.url.as_deref().unwrap_or("<unknown>"));
        } else if let Some(status) = tx.status {
            println!("Response: {} {}", status, tx.url.as_deref().unwrap_or("<unknown>"));
        }

        println!("Headers:");
        for (k, v) in &tx.headers {
            println!("  {}: {}", k, v);
        }

        if let Some(body) = &tx.body {
            println!("Body: {}", body);
        }
    }

    Ok(())
}
