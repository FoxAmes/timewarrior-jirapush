pub mod jira;

use futures::executor::block_on;
use jira::JiraWorklog;
use log::{debug, error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use std::{io::stdin, time::Duration};
use time::OffsetDateTime;

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

#[tokio::main]
async fn main() {
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
                            warn!(
                                "Unable to parse TimeWarrior config: {}, skipping",
                                stdin_line
                            );
                        }
                        _ => {
                            // Store valid lines
                            tw_conf.insert(
                                directives[0].to_string(),
                                directives[1..].join(":").to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    // Check required config (JIRA base URL, username, token)
    if !(tw_conf.contains_key("twjp.url")
        && tw_conf.contains_key("twjp.user")
        && tw_conf.contains_key("twjp.token"))
    {
        error!("Missing required config, please ensure twjp.url, twjp.user, and twjp.token are specified in your timewarrior config.");
        return;
    }

    // Set log level, if specified
    let mut log_level = log::LevelFilter::Warn;
    if tw_conf.contains_key("twjp.log_level") {
        match log::LevelFilter::from_str(&tw_conf["twjp.log_level"]) {
            Ok(l) => log_level = l,
            Err(_) => warn!("Invalid log level {}", tw_conf["twjp.log_level"]),
        }
    };

    // Build logger
    env_logger::builder().filter_level(log_level).init();

    // Read remaining JSON body of logs
    let tw_logs: Vec<TimeWarriorLog> = match serde_json::from_reader(stdin()) {
        Ok(l) => l,
        Err(e) => {
            error!("Error parsing timewarrior log as JSON: {}", e);
            return;
        }
    };

    // Iterate over logs
    let mut pending_logs = Vec::<(String, &TimeWarriorLog)>::new();
    for tw_log in &tw_logs {
        // Check if log is uploaded, and if not, if it's complete and so needs to be
        let is_uploaded = tw_log.tags.contains(&"jira-logged".to_string());
        let is_complete = tw_log.end.is_some();
        if !is_uploaded && is_complete {
            // Find a bugwarrior tag
            let bw_tag = tw_log.tags.iter().find(|t| t.starts_with("(bw)"));
            if let Some(bw_tag) = bw_tag {
                // See if tag contains a (valid-ish) Jira URL
                let url_re = Regex::new(r"(?P<url>https?://.+browse/(?P<issue>.+))").unwrap();
                let captures = url_re.captures(bw_tag);
                if let Some(c) = captures {
                    pending_logs.push((c["issue"].to_string(), tw_log));
                }
            }
        }
    }

    // If we have pending logs, construct a REST client and POST them
    if pending_logs.len() > 0 {
        // Build connection info
        let rest_c = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let jc = jira::JiraConnection {
            user: tw_conf["twjp.user"].clone(),
            token: tw_conf["twjp.token"].clone(),
            instance_url: tw_conf["twjp.url"].clone(),
        };

        // Handle our pending logs
        for (issue, log) in pending_logs {
            // Parse sparse ISO8601 dates handed off by TimeWarrior
            let start = OffsetDateTime::parse(
                log.start.to_string()
                    + tw_conf.get("twjp.timezone").unwrap_or(&"+0000".to_string()),
                "%Y%m%dT%H%M%SZ%z",
            )
            .unwrap();
            let end = OffsetDateTime::parse(
                log.end.as_ref().unwrap().clone()
                    + tw_conf.get("twjp.timezone").unwrap_or(&"+0000".to_string()),
                "%Y%m%dT%H%M%SZ%z",
            )
            .unwrap();

            // Construct a compatible Jira worklog
            let worklog = JiraWorklog {
                started: start.format("%FT%T.000%z"),
                time_spent_seconds: (end - start).whole_seconds(),
            };

            // Check to see if an existing worklog at that time exists (unless configured otherwise)
            if bool::from_str(
                tw_conf
                    .get("twjp.skip_existing")
                    .unwrap_or(&"true".to_string()),
            )
            .unwrap_or(true)
            {
                // Fetch existing logs
                let existing_logs = block_on(jira::get_worklogs(&rest_c, &jc, &issue));
                debug!("Existing logs: {:?}", existing_logs);
                // Compare logs
                let mut exists = false;
                for wl in existing_logs {
                    // Jira stores milliseconds which cannot be easily parsed here as there's no formatting directive
                    // We will superimpose 0's there so we can still parse.
                    let mut corrected_time = wl.started.clone();
                    corrected_time.replace_range(20..=22, "000");
                    let e_start =
                        OffsetDateTime::parse(corrected_time, "%Y-%m-%dT%H:%M:%S.000%z").unwrap();
                    if e_start == start {
                        exists = true;
                        break;
                    }
                }
                // We have a log here already, skip this one.
                if exists {
                    info!("Skipping existing log for {}", issue);
                    continue;
                }
            }
            // Upload
            match block_on(jira::upload_worklog(&rest_c, &jc, &issue, &worklog)) {
                Ok(_) => {
                    info!("Logged {:?} for {}", worklog, issue);
                }
                Err(e) => {
                    warn!("Error logging {:?} for {}: {}", worklog, issue, e);
                }
            }

            // Need to tag the log as uploaded, however we can't do that via a `timew` call as it will recursively call this extension.
        }
    }
}
