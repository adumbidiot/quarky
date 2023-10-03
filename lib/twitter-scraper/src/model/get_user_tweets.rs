use super::{
    ResultField,
    UserField,
};

pub type GetUserTweetsResponse = UserField<ResultField<UserResult>>;

#[derive(Debug, serde::Deserialize)]
pub struct UserResult {
    pub timeline_v2: TimelineV2,
}

#[derive(Debug, serde::Deserialize)]
pub struct TimelineV2 {
    pub timeline: Timeline,
}

#[derive(Debug, serde::Deserialize)]
pub struct Timeline {
    pub instructions: Vec<TimelineInstruction>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TimelineInstruction {
    #[serde(rename = "TimelineClearCache")]
    ClearCache,

    #[serde(rename = "TimelineAddEntries")]
    AddEntries { entries: Vec<TimelineEntry> },

    #[serde(rename = "TimelinePinEntry")]
    PinEntry,

    #[serde(rename = "TimelineTerminateTimeline")]
    TerminateTimeline,
}

#[derive(Debug, serde::Deserialize)]
pub struct TimelineEntry {
    #[serde(rename = "entryId")]
    pub entry_id: String,
}
