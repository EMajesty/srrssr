use chrono::DateTime;
use rss::Channel;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use terminal_size::{terminal_size, Height, Width};

#[tokio::main]
async fn main() {
    let mut all_items = Vec::new();

    let path = Path::new("sources");
    let file = File::open(&path).unwrap();
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(content) => match get_feed(content).await {
                // Ok(v) => println!("{}", v),
                Ok(v) => all_items.extend(v.items),
                Err(e) => println!("{}", e),
            },
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    all_items.sort_by(|a, b| {
        let date_a =
            DateTime::parse_from_rfc2822(a.pub_date().unwrap_or("")).unwrap_or_else(|_| {
                DateTime::parse_from_rfc2822("Thu, 01 Jan 1970 00:00:00 +0000").unwrap()
            });
        let date_b =
            DateTime::parse_from_rfc2822(b.pub_date().unwrap_or("")).unwrap_or_else(|_| {
                DateTime::parse_from_rfc2822("Thu, 01 Jan 1970 00:00:00 +0000").unwrap()
            });
        date_b.cmp(&date_a)
    });

    for item in all_items {
        println!(
            "{} - {}",
            item.pub_date().unwrap_or("Unknown date"),
            make_clickable_link(
                item.title().unwrap_or("No title"),
                item.link().unwrap_or("No link")
            ) // item.title().unwrap_or("No title"),
              // item.link().unwrap_or("No link")
        );
    }
}

async fn get_feed(url: String) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

fn make_clickable_link(text: &str, url: &str) -> String {
    let width:usize;
    if let Some((Width(w), Height(_))) = terminal_size() {
        width = w.into();
    } else {
        width = 50;
    }

    let truncated_text = truncate(text, width - 32);
    format!("\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\", url, truncated_text)
}

fn truncate(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length - 3])
    }
}
