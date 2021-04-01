use std::io::{stdin, Read};
use std::{collections::HashMap, env};
use time::OffsetDateTime;

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A structure representing a single TimeWarrior log entry.
/// # Examples
/// These logs come from TimeWarrior as a JSON object:
/// ```
/// {
///   "start":"20160405T162205Z",
///   "end":"20160405T162211Z",
///   "tags":["This is a multi-word tag","ProjectA","tag123"]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug)]
struct TimeWarriorLog {
    pub start: String,
    pub end: Option<String>,
    pub tags: Vec<String>,
}

fn main() {
    // Parse config initially passed from TimeWarrior
    let mut tw_conf = HashMap::<String, String>::new();
    let mut stdin_block = String::new();
    // Read and store config pairs
    while let Ok(chars) = stdin().read_line(&mut stdin_block) {
        match chars {
            0..=1 => {
                // If we've reached an empty line, there is no more config to parse
                break;
            }
            _ => {
                for stdin_line in stdin_block.split('\n').filter(|d| d.contains(":")) {
                    // Attempt to split the key-value pair
                    let directives: Vec<&str> = stdin_line.split(':').map(|d| d.trim()).collect();
                    match directives.len() {
                        0..=1 => {
                            // Skip invalid lines
                            println!(
                                "Unable to parse TimeWarrior config: {}, skipping",
                                stdin_line
                            );
                        }
                        _ => {
                            // Store valid lines
                            tw_conf.insert(directives[0].to_string(), directives[1].to_string());
                        }
                    }
                }
            }
        }
    }

    // Read remaining JSON body of logs
    let tw_logs: Vec<TimeWarriorLog> = match serde_json::from_reader(stdin()) {
        Ok(l) => l,
        Err(e) => panic!("Error parsing timewarrior log as JSON: {}", e),
    };

    // Iterate over logs to see which have been tagged as uploaded
    for tw_log in &tw_logs {
        let is_uploaded = tw_log.tags.contains(&"jira-logged".to_string());
        if !is_uploaded {
            // Find a bugwarrior tag
            let bw_tag = tw_log.tags.iter().find(|t| t.starts_with("(bw)"));
            if let Some(bw_tag) = bw_tag {
                // See if tag contains a (valid-ish) Jira URL
                let url_re = Regex::new(r"(?P<url>https?://.+browse/(?P<ticket>.+))").unwrap();
                let captures = url_re.captures(bw_tag);
                match captures {
                    Some(c) => {
                        println!(
                            "Need to upload log for {} at {}",
                            c["ticket"].to_string(),
                            c["url"].to_string()
                        )
                        // Haha upload it or something idk :)
                    }
                    None => {}
                }
            }
        }
    }

    // FIXME: Debug print
    //println!("Config: {:?}", tw_conf);
    //println!("Logs: {:?}", tw_logs);
}
