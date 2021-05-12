use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct TimeWarriorLog {
    pub id: usize,
    pub start: String,
    pub end: Option<String>,
    pub tags: Vec<String>,
}

/// Takes given TimeWarrior input and parses config and logs from it.
/// Returns a HashMap with configuration and a list of logs as a tuple.
pub fn parse_tw_input(
    input: &str,
) -> Result<(HashMap<String, String>, Vec<TimeWarriorLog>), String> {
    // Everything up to the first blank line is config
    let config_block = input
        .lines()
        .take_while(|l| l.trim().len() > 1)
        .collect::<Vec<&str>>()
        .join("\n");
    // Everything after is the JSON entry body
    let entries = input
        .lines()
        .skip(config_block.lines().count() + 1)
        .collect::<Vec<&str>>()
        .join("\n");

    // Parse config initially passed from TimeWarrior
    let tw_conf = parse_tw_config(&config_block);

    // Read remaining JSON body of logs
    #[derive(Serialize, Deserialize)]
    struct TimeWarriorLogRaw {
        pub id: Option<usize>,
        pub start: String,
        pub end: Option<String>,
        pub tags: Vec<String>,
    }
    debug!("Parsing entries: {:?}", entries);
    let mut tw_logs: Vec<TimeWarriorLogRaw> = match serde_json::from_str(&entries) {
        Ok(l) => l,
        Err(e) => {
            return Err(format!("Error parsing timewarrior log as JSON: {}", e));
        }
    };
    // Convert raw TW logs
    // We reverse the logs as they are in descending order by default
    tw_logs.reverse();
    let mut tw_logs: Vec<TimeWarriorLog> = tw_logs
        .into_iter()
        .enumerate()
        .filter_map(|(i, l)| {
            match l.id {
                Some(id) => Some(TimeWarriorLog {
                    id: id,
                    start: l.start,
                    end: l.end,
                    tags: l.tags,
                }),
                None => {
                    // For old TimeWarrior versions, we need to infer the ID
                    match semver::Version::parse(
                        tw_conf.get("temp.version").unwrap_or(&"1.0.0".to_string()),
                    )
                    .expect("Unable to parse TimeWarrior version")
                        < semver::Version::parse("1.3.0").unwrap()
                    {
                        true => {
                            warn!("You are using an outdated version of Timewarrior. Work logs may not be accurately identified.");
                            Some(TimeWarriorLog {
                                id: i + 1,
                                start: l.start,
                                end: l.end,
                                tags: l.tags,
                            })
                        },
                        false => None,
                    }
                }
            }
        })
        .collect();
    tw_logs.reverse();

    Ok((tw_conf, tw_logs))
}

/// Parses key-value configuration passed by TimeWarrior.
/// See https://timewarrior.net/docs/api/#input-format for more information.
fn parse_tw_config(block: &str) -> HashMap<String, String> {
    // Create a new map
    let mut tw_conf = HashMap::<String, String>::new();
    // Read and store config pairs
    for stdin_line in block.split('\n').filter(|d| d.contains(":")) {
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
    tw_conf
}

/// Tag a timewarrior interval
pub fn tag_tw_log(tw_log: &TimeWarriorLog, tag: &str) -> Result<(), String> {
    // Call the interval as uploaded
    match std::process::Command::new("timew")
        .args(&["tag", &format!("@{}", tw_log.id), tag])
        .output()
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "Error marking interval {} as uploaded: {}",
            tw_log.id, e
        )),
    }
}
