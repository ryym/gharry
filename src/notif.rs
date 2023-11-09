mod direct_review_request;
mod issue_close;
mod issue_comment;
mod plain;
mod pr_open;
mod pr_review;
mod pr_review_comment;
mod push;
mod team_review_request;
mod workflow_cancelled;

use crate::{email::Email, github, slack};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Notification {
    pub detail: NotifDetail,
}

#[derive(Debug, Serialize)]
pub enum NotifDetail {
    Unknown {
        sender: String,
        body: Vec<String>,
    },
    PrOpened {
        opener: github::User,
        pr: github::IssueInfo,
    },
    PrReviewed {
        url: String,
        pr: github::IssueInfo,
        state: github::ReviewState,
        commenter: github::User,
        comment: String,
    },
    PrReviewCommented {
        url: String,
        pr: github::IssueInfo,
        commenter: github::User,
        comment: String,
    },
    DirectReviewRequested {
        reviewee: github::User,
        pr: github::IssueInfo,
    },
    TeamReviewRequested {
        reviewee: github::User,
        pr: github::IssueInfo,
        team: String,
    },
    IssueClosed {
        closer: github::User,
        issue: github::IssueInfo,
        is_merge: bool,
    },
    Commented {
        url: String,
        issue: github::IssueInfo,
        commenter: github::User,
        comment: String,
    },
    Pushed {
        pr: github::IssueInfo,
        diff_url: String,
        committer: github::User,
        commits: Vec<github::CommitInfo>,
    },
    WorkflowCancelled {
        sender_name: String,
        repo_fullname: String,
        workflow_name: String,
        result_url: String,
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
            github::build_notif_from_email(&email)
                .and_then(|enotif| {
                    let n = Parser::parse(&cx, email, enotif)?;
                    Ok(Notification { detail: n.detail })
                })
                .map(|notif| {
                    notifs.push(notif);
                    notifs
                })
        })?;
    Ok(notifs)
}

const PARSERS: [Parser; 9] = [
    Parser::PrOpen,
    Parser::PrReview,
    Parser::PrReviewComment,
    Parser::DirectReviewRequest,
    Parser::TeamReviewRequest,
    Parser::IssueClosed,
    Parser::IssueComment,
    Parser::Push,
    Parser::WorkflowCancelled,
];

#[derive(Debug)]
enum Parser {
    PrOpen,
    PrReview,
    PrReviewComment,
    DirectReviewRequest,
    TeamReviewRequest,
    IssueClosed,
    IssueComment,
    Push,
    WorkflowCancelled,
}

impl Parser {
    fn parse(cx: &BuildContext, email: Email, enotif: github::EmailNotif) -> Result<Notification> {
        for p in PARSERS {
            if let Some(notif) = p.try_parse(cx, &email, &enotif)? {
                return Ok(notif);
            }
        }
        Ok(plain::parse(email, enotif))
    }

    fn try_parse(
        &self,
        cx: &BuildContext,
        email: &Email,
        enotif: &github::EmailNotif,
    ) -> Result<Option<Notification>> {
        match *self {
            Self::PrOpen => pr_open::try_parse(cx, enotif),
            Self::PrReview => pr_review::try_parse(cx, enotif),
            Self::PrReviewComment => pr_review_comment::try_parse(cx, enotif),
            Self::DirectReviewRequest => direct_review_request::try_parse(cx, enotif),
            Self::TeamReviewRequest => team_review_request::try_parse(cx, enotif),
            Self::IssueClosed => issue_close::try_parse(cx, enotif),
            Self::IssueComment => issue_comment::try_parse(cx, enotif),
            Self::Push => push::try_parse(cx, enotif),
            Self::WorkflowCancelled => workflow_cancelled::try_parse(email, enotif),
        }
    }
}
