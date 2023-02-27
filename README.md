# TimeWarrior Jira Push

## Summary

TimeWarrior-JiraPush (TWJP) is a configurable [TimeWarrior Extension](https://timewarrior.net/docs/api/) that uploads timewarrior intervals to Jira as work logs. It does not associate intervals with Jira tasks by itself, and requires a Jira issue URL in the tags for an interval (use a tool such as [BugWarrior](https://github.com/ralphbean/bugwarrior) to do this automatically) to work properly.

TWJP will tag timewarrior intervals when uploaded to reduce the number of API calls in subsequent runs, and check existing Jira worklogs for overlapping intervals when uploading logs to avoid duplicate uploads.

## Warning

TWJP is in early development, and mostly exists because I can't stand using Jira's web interface to upload time logs, so beyond basic functionality I may not have time to flesh out, fix, or otherwise support the tool. Use at your own risk!

## Installation and Usage

Build with `cargo build --release`, and place the compiled binary in your TimeWarrior extensions directory (likely `~/.timewarrior/extensions`).

Assuming the default binary name of `jirapush`, you can invoke the extension via `timew jirapush`, or any left-matched equivalend, such as `timew jira`.

## Configuration

### Example configuration

```
twjp.url = https://myjira.atlassian.net
twjp.user = user@example.com
twjp.token = my_access_token
twjp.log_level = warn
twjp.skip_existing = true
twjp.uploaded_tag = jira-uploaded
twjp.timezone = +0000
```

Configuration is specified in your `timewarrior.cfg`. Required are `twjp.url`, `twjp.user`, and `twjp.token`, as without these, the tool cannot connect to a Jira instance and therefore can't do anything.

### Configuration values

| key                | description                                                                                                                                             |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| twjp.url           | The base URL of your Jira instance                                                                                                                      |
| twjp.user          | Your Jira username/email                                                                                                                                |
| twjp.token         | An [API token](https://support.atlassian.com/atlassian-account/docs/manage-api-tokens-for-your-atlassian-account/) for the user                         |
| twjp.is_pat        | Treats `token` as a [personal access token (PAT)](https://confluence.atlassian.com/enterprise/using-personal-access-tokens-1026032365.html)             |
| twjp.log_level     | The log verbosity; one of `trace`, `debug`, `info`, `warn` (default), `error`, or `off`                                                                 |
| twjp.skip_existing | Unless set to `false`, will query Jira for existing work logs to avoid duplicate uploads. This does not affect skipping logs tagged locally as uploaded |
| twjp.uploaded_tag  | The tag to use when marking time intervals as uploaded. Defaults to `jira-uploaded`                                                                     |
| twjp.timezone      | The timezone offset to use when reading logs from TimeWarrior. Defaults to `+0000`, which is UTC                                                        |
