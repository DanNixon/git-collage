use chrono::{DateTime, FixedOffset, NaiveDateTime};
use git2::Time;
use url::Url;

pub(crate) fn safe_display_url(mut url: Url) -> String {
    if url.password() != None {
        let _ = url.set_password(Some("REDACTED"));
    }
    url.to_string()
}

pub(crate) fn git_timestamp(t: Time) -> DateTime<FixedOffset> {
    DateTime::from_utc(
        NaiveDateTime::from_timestamp(t.seconds(), 0),
        FixedOffset::east(t.offset_minutes() * 60),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_url() {
        let url = Url::parse("https://github.com/DanNixon/git-collage").unwrap();
        assert_eq!(
            "https://github.com/DanNixon/git-collage",
            safe_display_url(url)
        );
    }

    #[test]
    fn dirty_url() {
        let url = Url::parse("https://DanNixon:supersecrettoken@github.com/DanNixon/git-collage")
            .unwrap();
        assert_eq!(
            "https://DanNixon:REDACTED@github.com/DanNixon/git-collage",
            safe_display_url(url)
        );
    }
}
