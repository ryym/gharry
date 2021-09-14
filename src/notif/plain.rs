use crate::{email::Email, github, notif};

pub(super) fn parse(email: Email, enotif: github::EmailNotif) -> notif::Notification {
    notif::Notification {
        detail: notif::NotifDetail::Unknown {
            sender: email.sender_name,
            body: enotif.lines,
        },
    }
}
