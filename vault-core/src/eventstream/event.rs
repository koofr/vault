use serde::{Deserialize, Serialize};

use crate::{
    remote::models::FilesFile,
    types::{MountId, RemotePath},
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "fileCreated")]
    FileCreatedEvent {
        #[serde(rename = "mountId")]
        mount_id: MountId,
        path: RemotePath,
        file: FilesFile,
        #[serde(rename = "userAgent")]
        user_agent: Option<String>,
    },

    #[serde(rename = "fileRemoved")]
    FileRemovedEvent {
        #[serde(rename = "mountId")]
        mount_id: MountId,
        path: RemotePath,
        file: FilesFile,
        #[serde(rename = "userAgent")]
        user_agent: Option<String>,
    },

    #[serde(rename = "fileCopied")]
    FileCopiedEvent {
        #[serde(rename = "mountId")]
        mount_id: MountId,
        path: RemotePath,
        #[serde(rename = "newPath")]
        new_path: RemotePath,
        file: FilesFile,
        #[serde(rename = "userAgent")]
        user_agent: Option<String>,
    },

    #[serde(rename = "fileMoved")]
    FileMovedEvent {
        #[serde(rename = "mountId")]
        mount_id: MountId,
        path: RemotePath,
        #[serde(rename = "newPath")]
        new_path: RemotePath,
        file: FilesFile,
        #[serde(rename = "userAgent")]
        user_agent: Option<String>,
    },

    #[serde(rename = "fileTagsUpdated")]
    FileTagsUpdatedEvent {
        #[serde(rename = "mountId")]
        mount_id: MountId,
        path: RemotePath,
        file: FilesFile,
        #[serde(rename = "userAgent")]
        user_agent: Option<String>,
    },

    #[serde(rename = "fileRefreshed")]
    FileRefreshedEvent {
        #[serde(rename = "mountId")]
        mount_id: MountId,
        path: RemotePath,
        #[serde(rename = "userAgent")]
        user_agent: Option<String>,
    },

    #[serde(rename = "fileSyncDone")]
    FileSyncDoneEvent {
        #[serde(rename = "mountId")]
        mount_id: MountId,
        path: RemotePath,
        #[serde(rename = "userAgent")]
        user_agent: Option<String>,
    },

    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        remote::models::FilesFile,
        types::{MountId, RemoteName, RemotePath},
    };

    use super::Event;

    #[test]
    fn test_event_json() {
        assert_eq!(
            serde_json::from_str::<Event>(
                r#"{"type":"fileCreated","mountId":"mid","path":"/","newPath":"","file":{"name":"n","type":"file","modified":1665147222729,"size":1,"contentType":"application/octet-stream","hash":"ad9a9a286a20bb915e16eea9b2405c77","tags":{}},"userAgent":"ua"}"#
            ).unwrap(),
            Event::FileCreatedEvent {
                mount_id: MountId("mid".into()),
                path: RemotePath("/".into()),
                file: FilesFile {
                    name: RemoteName("n".into()),
                    typ: String::from("file"),
                    modified: 1665147222729,
                    size: 1,
                    content_type: String::from("application/octet-stream"),
                    hash: Some(String::from("ad9a9a286a20bb915e16eea9b2405c77")),
                    tags: HashMap::new(),
                },
                user_agent: Some(String::from("ua"))
            }
        );

        assert_eq!(
            serde_json::from_str::<Event>(r#"{"type":"someOtherEvent", "key": "value"}"#).unwrap(),
            Event::Unknown,
        );
    }
}
