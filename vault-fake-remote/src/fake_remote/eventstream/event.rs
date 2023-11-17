use vault_core::types::RemotePath;

use crate::fake_remote::files::Path;

use super::Event;

pub fn event_subject_id<'a>(event: &'a Event) -> Option<&'a str> {
    match event {
        Event::FileCreatedEvent { mount_id, .. } => Some(&mount_id.0),
        Event::FileRemovedEvent { mount_id, .. } => Some(&mount_id.0),
        Event::FileCopiedEvent { mount_id, .. } => Some(&mount_id.0),
        Event::FileMovedEvent { mount_id, .. } => Some(&mount_id.0),
        Event::FileTagsUpdatedEvent { mount_id, .. } => Some(&mount_id.0),
        Event::FileRefreshedEvent { mount_id, .. } => Some(&mount_id.0),
        Event::FileSyncDoneEvent { mount_id, .. } => Some(&mount_id.0),
        Event::Unknown => None,
    }
}

pub fn event_relative_to(event: Event, root_path: &Path) -> Option<Event> {
    let relative = |path: String| Path(path).relative_to(root_path);

    match event {
        Event::FileCreatedEvent {
            mount_id,
            path,
            file,
            user_agent,
        } => {
            if let Some(path) = relative(path.0) {
                Some(Event::FileCreatedEvent {
                    mount_id,
                    path: RemotePath(path.0),
                    file,
                    user_agent,
                })
            } else {
                None
            }
        }
        Event::FileRemovedEvent {
            mount_id,
            path,
            file,
            user_agent,
        } => {
            if let Some(path) = relative(path.0) {
                Some(Event::FileRemovedEvent {
                    mount_id,
                    path: RemotePath(path.0),
                    file,
                    user_agent,
                })
            } else {
                None
            }
        }
        Event::FileCopiedEvent {
            mount_id,
            path,
            new_path,
            file,
            user_agent,
        } => {
            if let Some(new_path) = relative(new_path.0) {
                if let Some(path) = relative(path.0) {
                    Some(Event::FileCopiedEvent {
                        mount_id,
                        path: RemotePath(path.0),
                        new_path: RemotePath(new_path.0),
                        file,
                        user_agent,
                    })
                } else {
                    Some(Event::FileCreatedEvent {
                        mount_id,
                        path: RemotePath(new_path.0),
                        file,
                        user_agent,
                    })
                }
            } else {
                None
            }
        }
        Event::FileMovedEvent {
            mount_id,
            path,
            new_path,
            file,
            user_agent,
        } => {
            if let Some(new_path) = relative(new_path.0) {
                if let Some(path) = relative(path.0) {
                    Some(Event::FileMovedEvent {
                        mount_id,
                        path: RemotePath(path.0),
                        new_path: RemotePath(new_path.0),
                        file,
                        user_agent,
                    })
                } else {
                    Some(Event::FileCreatedEvent {
                        mount_id,
                        path: RemotePath(new_path.0),
                        file,
                        user_agent,
                    })
                }
            } else {
                if let Some(path) = relative(path.0) {
                    Some(Event::FileRemovedEvent {
                        mount_id,
                        path: RemotePath(path.0),
                        file,
                        user_agent,
                    })
                } else {
                    None
                }
            }
        }
        Event::FileTagsUpdatedEvent {
            mount_id,
            path,
            file,
            user_agent,
        } => {
            if let Some(path) = relative(path.0) {
                Some(Event::FileTagsUpdatedEvent {
                    mount_id,
                    path: RemotePath(path.0),
                    file,
                    user_agent,
                })
            } else {
                None
            }
        }
        Event::FileRefreshedEvent {
            mount_id,
            path,
            user_agent,
        } => {
            if let Some(path) = relative(path.0) {
                Some(Event::FileRefreshedEvent {
                    mount_id,
                    path: RemotePath(path.0),
                    user_agent,
                })
            } else {
                None
            }
        }
        Event::FileSyncDoneEvent {
            mount_id,
            path,
            user_agent,
        } => {
            if let Some(path) = relative(path.0) {
                Some(Event::FileRefreshedEvent {
                    mount_id,
                    path: RemotePath(path.0),
                    user_agent,
                })
            } else {
                None
            }
        }
        Event::Unknown => None,
    }
}
