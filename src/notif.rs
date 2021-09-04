mod issue_comment;
mod plain;
mod pr_open;

use crate::{github, slack};
use anyhow::Result;

#[derive(Debug)]
pub struct Notification {
    pub detail: NotifDetail,
}

#[derive(Debug)]
pub enum NotifDetail {
    Unknown {
        sender: String,
        body: Vec<String>,
    },
    PrOpened {
        opener: github::User,
        pr: github::IssueInfo,
    },
    Commented {
        url: String,
        commenter: github::User,
        issue: github::IssueInfo,
        comment: String,
    },
}

#[derive(Debug)]
pub struct BuildContext<'a> {
    pub github: &'a github::Client,
}

pub fn build_notifications(
    cx: BuildContext,
    messages: Vec<slack::Message>,
) -> Result<Vec<Notification>> {
    let notifs = messages
        .into_iter()
        .filter_map(slack::extract_email_from_message)
        .try_fold(Vec::new(), |mut notifs, email| {
            github::build_notif_from_email(email)
                .and_then(|enotif| {
                    let n = Parser::parse(&cx, enotif)?;
                    Ok(Notification { detail: n.detail })
                })
                .map(|notif| {
                    notifs.push(notif);
                    notifs
                })
        })?;
    Ok(notifs)
}

const PARSERS: [Parser; 2] = [Parser::PrOpen, Parser::IssueComment];

#[derive(Debug)]
enum Parser {
    PrOpen,
    IssueComment,
}

impl Parser {
    fn parse(cx: &BuildContext, enotif: github::EmailNotif) -> Result<Notification> {
        for p in PARSERS {
            if let Some(notif) = p.try_parse(cx, &enotif)? {
                return Ok(notif);
            }
        }
        Ok(plain::parse(enotif))
    }

    fn try_parse(
        &self,
        cx: &BuildContext,
        enotif: &github::EmailNotif,
    ) -> Result<Option<Notification>> {
        match *self {
            Self::PrOpen => pr_open::try_parse(cx, enotif),
            Self::IssueComment => issue_comment::try_parse(cx, enotif),
        }
    }
}
