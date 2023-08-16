use crate::fake_remote::files::Path;

use super::Event;

pub fn event_subject_id<'a>(event: &'a Event) -> Option<&'a str> {
    match event {
        Event::FileCreatedEvent { mount_id, .. } => Some(mount_id),
        Event::FileRemovedEvent { mount_id, .. } => Some(mount_id),
        Event::FileCopiedEvent { mount_id, .. } => Some(mount_id),
        Event::FileMovedEvent { mount_id, .. } => Some(mount_id),
        Event::FileTagsUpdatedEvent { mount_id, .. } => Some(mount_id),
        Event::FileRefreshedEvent { mount_id, .. } => Some(mount_id),
        Event::FileSyncDoneEvent { mount_id, .. } => Some(mount_id),
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
            if let Some(path) = relative(path) {
                Some(Event::FileCreatedEvent {
                    mount_id,
                    path: path.0,
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
            if let Some(path) = relative(path) {
                Some(Event::FileRemovedEvent {
                    mount_id,
                    path: path.0,
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
            if let Some(new_path) = relative(new_path) {
                if let Some(path) = relative(path) {
                    Some(Event::FileCopiedEvent {
                        mount_id,
                        path: path.0,
                        new_path: new_path.0,
                        file,
                        user_agent,
                    })
                } else {
                    Some(Event::FileCreatedEvent {
                        mount_id,
                        path: new_path.0,
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
            if let Some(new_path) = relative(new_path) {
                if let Some(path) = relative(path) {
                    Some(Event::FileMovedEvent {
                        mount_id,
                        path: path.0,
                        new_path: new_path.0,
                        file,
                        user_agent,
                    })
                } else {
                    Some(Event::FileCreatedEvent {
                        mount_id,
                        path: new_path.0,
                        file,
                        user_agent,
                    })
                }
            } else {
                if let Some(path) = relative(path) {
                    Some(Event::FileRemovedEvent {
                        mount_id,
                        path: path.0,
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
            if let Some(path) = relative(path) {
                Some(Event::FileTagsUpdatedEvent {
                    mount_id,
                    path: path.0,
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
            if let Some(path) = relative(path) {
                Some(Event::FileRefreshedEvent {
                    mount_id,
                    path: path.0,
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
            if let Some(path) = relative(path) {
                Some(Event::FileRefreshedEvent {
                    mount_id,
                    path: path.0,
                    user_agent,
                })
            } else {
                None
            }
        }
        Event::Unknown => None,
    }
}
