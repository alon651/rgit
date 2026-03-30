use crate::structures::object::{Object, ObjectType};
use anyhow::{Context, ensure};
use chrono::{DateTime, Local};
use std::fmt;

/// A Git-style tag object.
#[derive(Debug, Clone)]
pub struct Tag {
    pub object: String,
    pub object_type: ObjectType,
    pub tag_name: String,
    pub tagger_name: String,
    pub tagger_email: String,
    pub timestamp: DateTime<Local>,
    pub message: Option<String>,
}

impl Tag {
    pub fn new(
        object: &str,
        object_type: ObjectType,
        tag_name: &str,
        tagger_name: &str,
        tagger_email: &str,
        timestamp: DateTime<Local>,
        message: Option<String>,
    ) -> Self {
        Self {
            object: object.to_string(),
            object_type,
            tag_name: tag_name.to_string(),
            tagger_name: tagger_name.to_string(),
            tagger_email: tagger_email.to_string(),
            timestamp,
            message,
        }
    }

    pub fn to_object(&self) -> Object {
        Object::new(self.to_string().into_bytes(), ObjectType::Tag)
    }

    #[allow(dead_code)]
    pub fn from_object(object: &Object) -> anyhow::Result<Self> {
        ensure!(object.object_type == ObjectType::Tag, "Object must be Tag, got {}",object.object_type);


        let (headers, message) = Object::parse_key_value(&object.data)?;

        // get the first value of a header or error
        let get_required = |key| {
            headers
                .get(key)
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("missing {} in tag", key))
        };

        let object = get_required("object")?.to_string();

        let object_type: ObjectType = ObjectType::from_str(get_required("type")?)?;

        let tag_name = get_required("tag")?.to_string();

        let tagger_line = get_required("tagger")?;

        let (tagger_name, tagger_email, timestamp) = parse_user_line(tagger_line, "tagger")?;

        Ok(Self {
            object,
            object_type,
            tag_name,
            tagger_name,
            tagger_email,
            timestamp,
            message,
        })
    }
}

fn parse_user_line(
    author_line: &str,
    arg: &str,
) -> anyhow::Result<(String, String, DateTime<Local>)> {
    let parts: Vec<&str> = author_line.splitn(3, ' ').collect();
    if parts.len() != 3 {
        anyhow::bail!("invalid {} line format in tag object", arg);
    }

    let timestamp_str = parts[2];

    let email = parts[1].trim_matches(|c| c == '<' || c == '>');
    let name = parts[0];
    let timestamp = DateTime::parse_from_str(timestamp_str, "%s %z")
        .context(format!("invalid timestamp: {} in {}", timestamp_str, arg))?
        .with_timezone(&Local);

    Ok((name.to_string(), email.to_string(), timestamp))
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut body = String::new();

        body.push_str(&format!("object {}\n", self.object));

        body.push_str(&format!("type {}\n", self.object_type));

        body.push_str(&format!("tag {}\n", self.tag_name));

        let ts = self.timestamp.format("%s %z");
        body.push_str(&format!(
            "tagger {} <{}> {}\n\n",
            self.tagger_name, self.tagger_email, ts
        ));

        if let Some(message) = &self.message {
            body.push_str(&format!("{}\n", message));
        }

        write!(f, "{}", body)
    }
}
