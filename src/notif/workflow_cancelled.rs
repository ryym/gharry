use crate::{github, notif};
use anyhow::Result;

pub(super) fn try_parse(
    email: &notif::Email,
    enotif: &github::EmailNotif,
) -> Result<Option<notif::Notification>> {
    if !email.subject.contains("Run cancelled:") {
        return Ok(None);
    }

    let mut repo_fullname: Option<String> = None;
    let mut workflow_name: Option<String> = None;
    let mut result_url: Option<String> = None;
    for l in &enotif.lines {
        match l.splitn(2, ":").collect::<Vec<&str>>()[..] {
            [name, value] => match name {
                "Repository" => repo_fullname = Some(value.trim().to_string()),
                "Workflow" => workflow_name = Some(value.trim().to_string()),
                "View results" => result_url = Some(value.trim().to_string()),
                _ => {}
            },
            _ => {}
        };
    }

    match (repo_fullname, workflow_name, result_url) {
        (Some(repo_fullname), Some(workflow_name), Some(result_url)) => {
            Ok(Some(notif::Notification {
                detail: notif::NotifDetail::WorkflowCancelled {
                    sender_name: email.sender_name.clone(),
                    repo_fullname,
                    workflow_name,
                    result_url,
                },
            }))
        }
        _ => Ok(None),
    }
}
