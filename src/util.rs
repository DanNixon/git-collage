use chrono::{DateTime, FixedOffset};
use git2::Time;
use url::Url;

pub(crate) fn safe_display_url(mut url: Url) -> String {
    if url.password().is_some() {
        let _ = url.set_password(Some("REDACTED"));
    }
    url.to_string()
}

pub(crate) fn git_timestamp(t: Time) -> DateTime<FixedOffset> {
    let tz = FixedOffset::east_opt(t.offset_minutes() * 60).unwrap();
    let t = DateTime::from_timestamp(t.seconds(), 0).unwrap();
    t.with_timezone(&tz)
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
