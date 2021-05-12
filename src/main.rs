#[cfg(test)]
pub(crate) mod tests;

pub mod jira;
pub mod timewarrior;

use jira::JiraWorklog;
use log::{debug, error, info, warn, LevelFilter};
use regex::{Captures, Regex};
use std::{io::stdin, io::Read, str::FromStr, time::Duration};
use time::OffsetDateTime;
use timewarrior::TimeWarriorLog;

fn main() {
    // Parse TimeWarrior input
    let mut input = String::new();
    stdin()
        .read_to_string(&mut input)
        .expect("Error reading from stdin");
    let (tw_conf, tw_logs) =
        timewarrior::parse_tw_input(&input).expect("Error parsing TimeWarrior input");

    // Check required config (JIRA base URL, username, token)
    if !(tw_conf.contains_key("twjp.url")
        && tw_conf.contains_key("twjp.user")
        && tw_conf.contains_key("twjp.token"))
    {
        error!("Missing required config, please ensure twjp.url, twjp.user, and twjp.token are specified in your timewarrior config.");
        return;
    }

    // Handle "well-behaved" config guidelines
    // https://timewarrior.net/docs/api/#guidelines
    let mut log_level = LevelFilter::Error;
    if let Some(val) = tw_conf.get("verbose") {
        match val.as_str() {
            "on" | "1" | "yes" | "y" | "true" => log_level = LevelFilter::Warn,
            _ => {}
        }
    }
    if let Some(val) = tw_conf.get("debug") {
        match val.as_str() {
            "on" | "1" | "yes" | "y" | "true" => log_level = LevelFilter::Debug,
            _ => {}
        }
    }

    // Set log level, if specified
    if let Some(val) = tw_conf.get("twjp.log_level") {
        match LevelFilter::from_str(val) {
            Ok(l) => log_level = l,
            Err(_) => warn!("Invalid log level {}", val),
        }
    };

    // Build logger
    env_logger::builder().filter_level(log_level).init();

    // Iterate over logs to determine work we need to do
    let upload_tag = tw_conf
        .get("twjp.uploaded_tag")
        .unwrap_or(&"jira-uploaded".to_string())
        .clone();
    let mut pending_logs = Vec::<(String, &TimeWarriorLog)>::new();
    for tw_log in &tw_logs {
        // Check if log is uploaded, and if not, if it's complete and so needs to be
        let is_uploaded = tw_log.tags.contains(&upload_tag);
        let is_complete = tw_log.end.is_some();
        if !is_uploaded && is_complete {
            // Find the first tag containing a Jira-esque URL
            let url_re = Regex::new(r"(?P<url>https?://.+browse/(?P<issue>.+))").unwrap();
            let url_tags = tw_log.tags.iter().filter_map(|t| url_re.captures(t));
            if let Some(url_captures) = url_tags.collect::<Vec<Captures>>().first() {
                pending_logs.push((url_captures["issue"].to_string(), tw_log));
            }
        }
    }

    // If we have pending logs, construct a REST client and POST them
    // Additionally, mark uploaded logs as such
    if pending_logs.len() > 0 {
        // Build connection info
        let rest_c = reqwest::blocking::Client::builder()
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
                let existing_logs = jira::get_worklogs(&rest_c, &jc, &issue);
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
                    // Tag the interval as uploaded
                    match timewarrior::tag_tw_log(&log, &upload_tag) {
                        Ok(_) => {
                            info!("Log already exists for {}, marking as uploaded.", issue);
                        }
                        Err(e) => {
                            warn!(
                                "Error marking existing interval {:?} as uploaded: {}",
                                log, e
                            );
                        }
                    }
                    continue;
                }
            }
            // Upload
            match jira::upload_worklog(&rest_c, &jc, &issue, &worklog) {
                Ok(_) => {
                    // Tag the interval as uploaded
                    match timewarrior::tag_tw_log(&log, &upload_tag) {
                        Ok(_) => {
                            info!("Logged for {}", issue);
                        }
                        Err(e) => {
                            warn!("Error marking interval {:?} as uploaded: {}", log, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error logging {:?} for {}: {}", worklog, issue, e);
                }
            }
        }
    }
}
