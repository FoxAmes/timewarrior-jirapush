use log::{debug, warn};
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};

/// Jira instance connection information
pub struct JiraConnection {
    pub user: String,
    pub token: String,
    pub instance_url: String,
}

/// A structure representing a Jira work log.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklog {
    pub started: String,
    pub time_spent_seconds: i64,
}

/// Generic get function for Jira API
fn get(
    rc: &Client,
    jc: &JiraConnection,
    endpoint: &str,
    query: &Vec<(String, String)>,
) -> reqwest::Result<Response> {
    rc.get(format!(
        "{base_url}/{endpoint}",
        base_url = jc.instance_url,
        endpoint = endpoint
    ))
    .basic_auth(&jc.user, Some(&jc.token))
    .header("Accept", "application/json")
    .query(query)
    .send()
}

/// Generic post function for Jira API
fn post(
    rc: &Client,
    jc: &JiraConnection,
    endpoint: &str,
    body: String,
) -> reqwest::Result<Response> {
    rc.post(format!(
        "{base_url}/{endpoint}",
        base_url = jc.instance_url,
        endpoint = endpoint
    ))
    .basic_auth(&jc.user, Some(&jc.token))
    .header("Accept", "application/json")
    .header("Content-Type", "application/json")
    .body(body)
    .send()
}

#[derive(Serialize, Deserialize)]
struct JiraResponseWorklog {
    worklogs: Vec<JiraWorklog>,
}

/// Pulls existing worklogs from Jira for a given issue.
pub fn get_worklogs(rc: &Client, jc: &JiraConnection, issue: &str) -> Vec<JiraWorklog> {
    // Fetch worklogs
    match get(
        rc,
        jc,
        &format!("rest/api/3/issue/{issue}/worklog", issue = issue),
        &vec![],
    ) {
        // Handle failed connections
        Err(e) => {
            warn!(
                "Connection error fetching worklogs for {issue}: {error}",
                issue = issue,
                error = e
            );
            vec![]
        }
        // On successful connection, ensure success
        Ok(r) => match r.status().is_success() {
            false => {
                warn!(
                    "Error fetching worklogs for {issue}: {status}",
                    issue = issue,
                    status = r.status()
                );
                vec![]
            }
            // On successful fetch, move on
            true => {
                let body = &r.text().unwrap().clone();
                let jr: JiraResponseWorklog = match serde_json::from_str(&body) {
                    Err(e) => {
                        warn!(
                            "Error parsing worklogs for {issue}: {error}",
                            issue = issue,
                            error = e
                        );
                        debug!("Body: {body}", body = &body);
                        JiraResponseWorklog { worklogs: vec![] }
                    }
                    Ok(r) => r,
                };
                jr.worklogs
            }
        },
    }
}

/// Uploads a worklog to Jira.
pub fn upload_worklog(
    rc: &Client,
    jc: &JiraConnection,
    issue: &str,
    wl: &JiraWorklog,
) -> Result<(), String> {
    let mut temp_wl = wl.clone();
    // Worklogs under 60 seconds are not recognized by JIRA, we need to round up
    if temp_wl.time_spent_seconds < 60 {
        temp_wl.time_spent_seconds = 60;
    }
    // Fetch worklogs
    match post(
        rc,
        jc,
        &format!("rest/api/3/issue/{issue}/worklog", issue = issue),
        serde_json::to_string(&temp_wl).unwrap(),
    ) {
        // Handle failed connections
        Err(e) => Err(format!(
            "Connection error uploading worklog for {issue}: {error}",
            issue = issue,
            error = e
        )),
        // On successful connection, check status code
        Ok(r) => match r.status().is_success() {
            false => Err(format!(
                "Error submitting worklog for {issue}: {status}\n{body:?}",
                issue = issue,
                status = r.status(),
                body = r.text().unwrap()
            )),
            // On successful fetch, move on
            true => Ok(()),
        },
    }
}
