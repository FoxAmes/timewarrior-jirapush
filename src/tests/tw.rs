pub(crate) mod tw13 {
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
        // Ensure successful parse
        let (_, _) = crate::parse_tw_input(&EXAMPLE_TW13_STDIN).unwrap();
    }

    #[test]
    fn validate_log_parsing_count() {
        // Ensure successful parse
        let (_, twl) = crate::parse_tw_input(&EXAMPLE_TW13_STDIN).unwrap();
        // Validate number of logs
        assert_eq!(twl.len(), 2);
    }

    #[test]
    fn validate_log_parsing_ids() {
        // Ensure successful parse
        let (_, twl) = crate::parse_tw_input(&EXAMPLE_TW13_STDIN).unwrap();
        // Validate log IDs
        assert_eq!(twl[0].id, 2);
        assert_eq!(twl[1].id, 1);
    }

    #[test]
    fn validate_log_parsing_times() {
        // Ensure successful parse
        let (_, twl) = crate::parse_tw_input(&EXAMPLE_TW13_STDIN).unwrap();
        // Validate start times
        assert_eq!(twl[0].start, "20210101T000000Z");
        assert_eq!(twl[1].start, "20210102T000000Z");
        // Validate end times
        assert_eq!(twl[0].end, Some("20210102T000000Z".to_string()));
        assert_eq!(twl[1].end, None);
    }
}

pub(crate) mod tw12 {
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
        // Ensure successful parse
        let (_, _) = crate::parse_tw_input(&EXAMPLE_TW12_STDIN).unwrap();
    }

    #[test]
    fn validate_log_parsing_count() {
        // Ensure successful parse
        let (_, twl) = crate::parse_tw_input(&EXAMPLE_TW12_STDIN).unwrap();
        // Validate number of logs
        assert_eq!(twl.len(), 2);
    }

    #[test]
    fn validate_log_parsing_ids() {
        // Ensure successful parse
        let (_, twl) = crate::parse_tw_input(&EXAMPLE_TW12_STDIN).unwrap();
        // Validate log IDs
        assert_eq!(twl[0].id, 2);
        assert_eq!(twl[1].id, 1);
    }

    #[test]
    fn validate_log_parsing_times() {
        // Ensure successful parse
        let (_, twl) = crate::parse_tw_input(&EXAMPLE_TW12_STDIN).unwrap();
        // Validate start times
        assert_eq!(twl[0].start, "20210101T000000Z");
        assert_eq!(twl[1].start, "20210102T000000Z");
        // Validate end times
        assert_eq!(twl[0].end, Some("20210102T000000Z".to_string()));
        assert_eq!(twl[1].end, None);
    }
}
