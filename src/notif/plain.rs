use crate::{github, notif};

pub(super) fn parse(enotif: github::EmailNotif) -> notif::Notification {
    notif::Notification {
        detail: notif::NotifDetail::Unknown {
            sender: enotif.email.sender_name,
            body: enotif.lines,
        },
    }
}
