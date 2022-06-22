# 🎠 Gharry

Gharry is a GitHub email notification resender optimized for **my** daily work.

## What I am doing

1. Configure GitHub to send notificaition emails to Slack's muted channel.
2. Run Gharry which scans email notifications via Slack API periodically.
3. Filter, parse and re-send notifications to Slack's unmuted channel.

```
GitHub -[email]-> Slack -[Gharry]-> Slack
```

## Why???

- Implementing my own notifier allows me to customize its behavior completely for my work. For example,
    - I want to unsubscribe some notifications automatically.
    - I want to change a notification level (presence or absense of `@mention`) per notification.
- I wanted to write something in Rust for the first time in a while.

## Implementation

- Do simple polling instead of event driven approach.
    - I don't have a permission to install webhooks to my organization.
    - The GitHub's notifications API cannot be used since it ignores review state changes.
- Do everything in a single thread with blocking. No asynchronous IO.
    - This is enough for this program.

## Screenshot

A sample image of notifications from `facebook/react` when I'm watching the repository:

![Slack screenshot](https://raw.githubusercontent.com/ryym/i/master/gharry/slack-sample.png)

## Usage

1. Slack setup
    - Create a bot to use Slack API.
    - Create two channels.
    - Configure email app and get an email address.
2. GitHub setup
    - Configure to send notifications to the email address created by Slack.
    - Generate an access token (scope: repo, notifications).
3. Gharry configuration
    - Put a config file in `~/.gharry/config.toml`.

    ```toml
    slack_oauth_bot_token = "..."
    slack_mail_channel_id = "..."
    slack_dest_channel_id = "..."
    github_access_token = "..."
    github_login_name = "ryym"
    ```
