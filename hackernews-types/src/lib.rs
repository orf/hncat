use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Item {
    DeletedItem {
        id: u32,
        deleted: bool,
        #[serde(with = "ts_seconds")]
        time: DateTime<Utc>,
        #[serde(rename = "type")]
        type_: String,
    },
    LiveItem {
        by: String,
        #[serde(default)]
        dead: bool,
        id: u32,
        #[serde(with = "ts_seconds")]
        time: DateTime<Utc>,
        #[serde(flatten)]
        variant: ItemVariant,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ItemVariant {
    Story {
        #[serde(default)]
        descendants: i32,
        #[serde(default)]
        kids: Vec<u32>,
        score: i32,
        title: String,
        // Ask-hn and stories are the same fucking structure. Urgh. Need to differencate this somehow.
        url: Option<String>,
        text: Option<String>,
    },
    Comment {
        #[serde(default)]
        kids: Vec<u32>,
        parent: u32,
        text: String,
    },
    Job {
        score: i32,
        text: Option<String>,
        title: String,
        url: Option<String>,
    },
    Poll {
        descendants: i32,
        #[serde(default)]
        kids: Vec<u32>,
        #[serde(default)]
        parts: Vec<u32>,
        score: i32,
        title: String,
    },
    PollOpt {
        poll: u32,
        score: i32,
        text: String,
    },
}

#[cfg(test)]
mod tests {
    use crate::Item;
    use matches::assert_matches;
    use serde_json;

    #[test]
    fn test_simple_parse() {
        let json = r#"
        {
          "by": "gzer0",
          "dead": false,
          "id": 27164354,
          "time": 1620568471,
          "type": "comment",
          "kids": [],
          "parent": 27164095,
          "text": "A message goes here"
        }
        "#;
        assert_matches!(serde_json::from_str(json), Ok(Item::LiveItem { .. }))
    }
}
