use log::{debug, warn};
use reqwest::{Client, Response, RequestBuilder};
use reqwest::header;
use serde::{Deserialize, Serialize};

/// Jira instance connection information
#[derive(Clone, Debug)]
pub struct JiraConnection {
    pub user: String,
    pub token: String,
    pub is_pat: bool,
    pub instance_url: String,
}

/// A structure representing a Jira work log.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JiraWorklog {
    pub started: String,
    pub time_spent_seconds: i64,
}

fn add_common_headers(
    rc: RequestBuilder,
    jc: &JiraConnection
) -> reqwest::RequestBuilder {
    let mut tmp = rc
        .header(header::ACCEPT, header::HeaderValue::from_static("application/json"))
        .header(header::USER_AGENT, header::HeaderValue::from_static("jirapush/0.1.0"));
    if jc.is_pat {
        //tmp = tmp.bearer_auth(&jc.token);
        tmp = tmp.header(header::AUTHORIZATION, format!("Bearer {}", &jc.token));
    } else {
        tmp = tmp.basic_auth(&jc.user, Some(&jc.token));
    }
    debug!("jc: {:#?}", &jc);
    debug!("req: {:#?}", &tmp);
    tmp
}

/// Generic get function for Jira API
async fn get(
    rc: &Client,
    jc: &JiraConnection,
    endpoint: &str,
    query: &Vec<(String, String)>,
) -> reqwest::Result<Response> {
    add_common_headers(rc.get(format!(
        "{base_url}/{endpoint}",
        base_url = jc.instance_url,
        endpoint = endpoint
    )), &jc)
    .query(query)
    .send()
    .await
}

/// Generic post function for Jira API
async fn post(
    rc: &Client,
    jc: &JiraConnection,
    endpoint: &str,
    body: String,
) -> reqwest::Result<Response> {
    add_common_headers(rc.post(format!(
        "{base_url}/{endpoint}",
        base_url = jc.instance_url,
        endpoint = endpoint
    )), &jc)
    .header("Content-Type", "application/json")
    .body(body)
    .send()
    .await
}

#[derive(Serialize, Deserialize)]
struct JiraResponseWorklog {
    worklogs: Vec<JiraWorklog>,
}

/// Pulls existing worklogs from Jira for a given issue.
pub async fn get_worklogs(rc: &Client, jc: &JiraConnection, issue: &str) -> Vec<JiraWorklog> {
    // Fetch worklogs
    match get(
        rc,
        jc,
        &format!("rest/api/latest/issue/{issue}/worklog", issue = issue),
        &vec![],
    )
    .await
    {
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
                let body = &r.text().await.unwrap().clone();
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
pub async fn upload_worklog(
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
        &format!("rest/api/latest/issue/{issue}/worklog", issue = issue),
        serde_json::to_string(&temp_wl).unwrap(),
    )
    .await
    {
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
                body = r.text().await.unwrap()
            )),
            // On successful fetch, move on
            true => Ok(()),
        },
    }
}
