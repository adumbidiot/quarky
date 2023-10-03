/// An RSS Feed
#[derive(Debug, serde::Deserialize)]
pub struct RssFeed {
    pub channel: Channel,
}

/// RSS Channel
#[derive(Debug, serde::Deserialize)]
pub struct Channel {
    /// The channel title
    pub title: ChannelTitle,

    /*
    /// The channel link
    pub link: ChannelLink,
    */
    /// A channel items
    pub item: Vec<ChannelItem>,
}

/// RSS Channel Title
#[derive(Debug, serde::Deserialize)]
pub struct ChannelTitle {
    /// The title
    #[serde(rename = "$text")]
    pub title: Box<str>,
}

/// RSS Channel Link
#[derive(Debug, serde::Deserialize)]
pub struct ChannelLink {
    /// The link
    #[serde(rename = "$text")]
    pub link: Box<str>,
}

/// RSS Channel Item
#[derive(Debug, serde::Deserialize)]
pub struct ChannelItem {
    /// A channel item link
    pub link: ChannelItemLink,
}

/// RSS Channel Item Url
#[derive(Debug, serde::Deserialize)]
pub struct ChannelItemLink {
    /// The link
    #[serde(rename = "$text")]
    pub urllink: Box<str>,
}
