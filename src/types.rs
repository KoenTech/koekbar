use chrono::Local;

pub struct StatusUpdate {
    pub time: chrono::DateTime<Local>,
    pub media: Option<MediaInfo>,
}

#[derive(Clone)]
pub struct MediaInfo {
    pub title: String,
    pub author: Option<String>,
    pub application: String,
}
