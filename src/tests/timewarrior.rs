use crate::timewarrior::*;

// Utility functions
fn validate_log_parsing(input: &str) {
    // Ensure successful parse
    let (_, _) = parse_tw_input(input).unwrap();
}

fn validate_log_parsing_count(input: &str) {
    // Ensure successful parse
    let (_, twl) = parse_tw_input(input).unwrap();
    // Validate number of logs
    assert_eq!(twl.len(), 2);
}

fn validate_log_parsing_ids(input: &str) {
    // Ensure successful parse
    let (_, twl) = parse_tw_input(input).unwrap();
    // Validate log IDs
    assert_eq!(twl[0].id, 2);
    assert_eq!(twl[1].id, 1);
}

fn validate_log_parsing_times(input: &str) {
    // Ensure successful parse
    let (_, twl) = parse_tw_input(input).unwrap();
    // Validate start times
    assert_eq!(twl[0].start, "20210101T000000Z");
    assert_eq!(twl[1].start, "20210102T000000Z");
    // Validate end times
    assert_eq!(twl[0].end, Some("20210102T000000Z".to_string()));
    assert_eq!(twl[1].end, None);
}

fn validate_config_size(input: &str) {
    // Ensure successful parse
    let (twc, _) = parse_tw_input(input).unwrap();
    // Validate config
    assert_eq!(twc.len(), 7);
}

fn validate_config_keys(input: &str) {
    // Ensure successful parse
    let (twc, _) = parse_tw_input(input).unwrap();
    // Validate config
    assert_eq!(twc["twjp.log_level"], "debug");
    assert_eq!(twc["twjp.skip_existing"], "true");
    assert_eq!(twc["twjp.token"], "secret");
    assert_eq!(twc["twjp.url"], "https://myjira.atlassian.net");
    assert_eq!(twc["twjp.user"], "user@myjira.com");
    assert_eq!(twc["verbose"], "on");
}

pub(crate) mod tw130 {
    const EXAMPLE_TW13_STDIN: &str = {
        r#"temp.version: 1.4.2
twjp.log_level: debug
twjp.skip_existing: true
twjp.token: secret
twjp.url: https://myjira.atlassian.net
twjp.user: user@myjira.com
verbose: on

[
{"id":2,"start":"20210101T000000Z","end":"20210102T000000Z","tags":["(bw)Is#2 - Example 2 .. https://myjira.atlassian.net/browse/ISSUE-2","ISSUE"]},
{"id":1,"start":"20210102T000000Z","tags":["(bw)Is#1 - Example 1 .. https://myjira.atlassian.net/browse/ISSUE-1","ISSUE"]}
]
"#
    };

    #[test]
    fn validate_log_parsing() {
        super::validate_log_parsing(&EXAMPLE_TW13_STDIN);
    }

    #[test]
    fn validate_log_parsing_count() {
        super::validate_log_parsing_count(&EXAMPLE_TW13_STDIN);
    }

    #[test]
    fn validate_log_parsing_ids() {
        super::validate_log_parsing_ids(&EXAMPLE_TW13_STDIN);
    }

    #[test]
    fn validate_log_parsing_times() {
        super::validate_log_parsing_times(&EXAMPLE_TW13_STDIN);
    }

    #[test]
    fn validate_config_size() {
        super::validate_config_size(&EXAMPLE_TW13_STDIN);
    }

    #[test]
    fn validate_config_keys() {
        super::validate_config_keys(&EXAMPLE_TW13_STDIN);
    }
}

pub(crate) mod tw120 {
    const EXAMPLE_TW12_STDIN: &str = {
        r#"temp.version: 1.2.0
twjp.log_level: debug
twjp.skip_existing: true
twjp.token: secret
twjp.url: https://myjira.atlassian.net
twjp.user: user@myjira.com
verbose: on

[
{"start":"20210101T000000Z","end":"20210102T000000Z","tags":["(bw)Is#2 - Example 2 .. https://myjira.atlassian.net/browse/ISSUE-2","ISSUE"]},
{"start":"20210102T000000Z","tags":["(bw)Is#1 - Example 1 .. https://myjira.atlassian.net/browse/ISSUE-1","ISSUE"]}
]
"#
    };

    #[test]
    fn validate_log_parsing() {
        super::validate_log_parsing(&EXAMPLE_TW12_STDIN);
    }

    #[test]
    fn validate_log_parsing_count() {
        super::validate_log_parsing_count(&EXAMPLE_TW12_STDIN);
    }

    #[test]
    fn validate_log_parsing_ids() {
        super::validate_log_parsing_ids(&EXAMPLE_TW12_STDIN);
    }

    #[test]
    fn validate_log_parsing_times() {
        super::validate_log_parsing_times(&EXAMPLE_TW12_STDIN);
    }

    #[test]
    fn validate_config_size() {
        super::validate_config_size(&EXAMPLE_TW12_STDIN);
    }

    #[test]
    fn validate_config_keys() {
        super::validate_config_keys(&EXAMPLE_TW12_STDIN);
    }
}
