use scraper::{
    ElementRef,
    Html,
    Selector,
};
use std::sync::LazyLock;

static TIMELINE_ITEM_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".timeline .timeline-item").unwrap());
static TWEET_LINK_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".tweet-link").unwrap());

/// An error that may occur while parsing from Html
#[derive(Debug, thiserror::Error)]
pub enum FromHtmlError {
    #[error("missing {0} element")]
    MissingElement(&'static str),
}

#[derive(Debug)]
pub struct Feed {
    pub timeline_items: Vec<TimelineItem>,
}

impl Feed {
    /// Parse from html.
    pub(crate) fn from_html(html: &Html) -> Result<Self, FromHtmlError> {
        let timeline_items = html
            .select(&TIMELINE_ITEM_SELECTOR)
            .map(TimelineItem::from_element)
            .collect::<Result<_, _>>()?;

        Ok(Self { timeline_items })
    }
}

#[derive(Debug)]
pub struct TimelineItem {
    pub link: String,
}

impl TimelineItem {
    fn from_element(element: ElementRef) -> Result<Self, FromHtmlError> {
        let link = element
            .select(&TWEET_LINK_SELECTOR)
            .next()
            .and_then(|el| el.attr("href"))
            .ok_or(FromHtmlError::MissingElement("tweet-link"))?;

        Ok(Self {
            link: link.to_string(),
        })
    }
}
