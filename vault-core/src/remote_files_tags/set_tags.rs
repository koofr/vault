use std::collections::{HashMap, HashSet};

use crate::remote::remote::RemoteFileTagsSetConditions;

pub fn set_tags(
    file_size: Option<i64>,
    file_modified: Option<i64>,
    file_hash: Option<&str>,
    file_tags: &mut HashMap<String, Vec<String>>,
    patch_tags: HashMap<String, Vec<String>>,
    conditions: &RemoteFileTagsSetConditions,
) -> Result<(), String> {
    if let Some(if_size) = conditions.if_size {
        if Some(if_size) != file_size {
            return Err("Set tags if size does not match".into());
        }
    }
    if let Some(if_modified) = conditions.if_modified {
        if Some(if_modified) != file_modified {
            return Err("Set tags if modified does not match".into());
        }
    }
    if let Some(if_hash) = conditions.if_hash.as_deref() {
        if Some(if_hash) != file_hash {
            return Err("Set tags if hash does not match".into());
        }
    }
    if let Some(if_old_tags) = &conditions.if_old_tags {
        for (if_key, if_values) in if_old_tags {
            let old_values = file_tags.get(if_key);

            if if_values.is_empty() {
                if old_values.is_some() {
                    return Err(format!(
                        "Set tags if old tags expected empty tags for key: {}",
                        if_key
                    ));
                }
            } else {
                match old_values {
                    Some(old_values) => {
                        if old_values.len() != if_values.len() {
                            return Err(format!("Set tags if old tags number of old tags does not match the expected tags: {} != {}", old_values.len(), if_values.len()));
                        }

                        let old_values_set = old_values.iter().collect::<HashSet<_>>();

                        for if_value in if_values {
                            if !old_values_set.contains(if_value) {
                                return Err(format!("Set tags if old tags old value does not match the new value: {}", if_key));
                            }
                        }
                    }
                    None => {
                        return Err(format!(
                            "Set tags if old tags expected non-empty tags for key: {}",
                            if_key
                        ))
                    }
                }
            }
        }
    }

    for (key, new_values) in patch_tags {
        if new_values.is_empty() {
            file_tags.remove(&key);
        } else {
            file_tags.insert(key, new_values);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use similar_asserts::assert_eq;

    use crate::remote::remote::RemoteFileTagsSetConditions;

    use super::set_tags;

    fn example_tags() -> HashMap<String, Vec<String>> {
        HashMap::from([
            ("t1".into(), vec!["v1".into(), "v2".into()]),
            ("t2".into(), vec!["v3".into()]),
        ])
    }

    #[test]
    fn test_set_tags() {
        let mut file_tags = HashMap::new();

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            example_tags(),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(file_tags, example_tags());

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            HashMap::from([
                ("t2".into(), vec!["v4".into()]),
                ("t3".into(), vec!["v5".into()]),
            ]),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(
            file_tags,
            HashMap::from([
                ("t1".into(), vec!["v1".into(), "v2".into()]),
                ("t2".into(), vec!["v4".into()]),
                ("t3".into(), vec!["v5".into()])
            ])
        );

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            HashMap::from([
                ("t1".into(), vec!["v1".into(), "v2".into()]),
                ("t3".into(), vec![]),
            ]),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(
            file_tags,
            HashMap::from([
                ("t1".into(), vec!["v1".into(), "v2".into()]),
                ("t2".into(), vec!["v4".into()])
            ])
        );

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            HashMap::from([
                ("t1".into(), vec![]),
                ("t2".into(), vec![]),
                ("t3".into(), vec![]),
            ]),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(file_tags, HashMap::new());
    }

    #[test]
    fn test_set_tags_conditions_if_size() {
        let mut file_tags = HashMap::new();

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            example_tags(),
            &RemoteFileTagsSetConditions {
                if_size: Some(1),
                if_modified: None,
                if_hash: None,
                if_old_tags: None,
            },
        )
        .unwrap();
        assert_eq!(file_tags, example_tags());

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                example_tags(),
                &RemoteFileTagsSetConditions {
                    if_size: Some(10),
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: None,
                },
            )
            .unwrap_err(),
            "Set tags if size does not match".to_string()
        );

        assert_eq!(
            set_tags(
                None,
                Some(2),
                Some("h1"),
                &mut file_tags,
                example_tags(),
                &RemoteFileTagsSetConditions {
                    if_size: Some(10),
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: None,
                },
            )
            .unwrap_err(),
            "Set tags if size does not match".to_string()
        );
    }

    #[test]
    fn test_set_tags_conditions_if_modified() {
        let mut file_tags = HashMap::new();

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            example_tags(),
            &RemoteFileTagsSetConditions {
                if_size: None,
                if_modified: Some(2),
                if_hash: None,
                if_old_tags: None,
            },
        )
        .unwrap();
        assert_eq!(file_tags, example_tags());

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                example_tags(),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: Some(20),
                    if_hash: None,
                    if_old_tags: None,
                },
            )
            .unwrap_err(),
            "Set tags if modified does not match".to_string()
        );

        assert_eq!(
            set_tags(
                Some(1),
                None,
                Some("h1"),
                &mut file_tags,
                example_tags(),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: Some(20),
                    if_hash: None,
                    if_old_tags: None,
                },
            )
            .unwrap_err(),
            "Set tags if modified does not match".to_string()
        );
    }

    #[test]
    fn test_set_tags_conditions_if_hash() {
        let mut file_tags = HashMap::new();

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            example_tags(),
            &RemoteFileTagsSetConditions {
                if_size: None,
                if_modified: None,
                if_hash: Some("h1".into()),
                if_old_tags: None,
            },
        )
        .unwrap();
        assert_eq!(file_tags, example_tags());

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                example_tags(),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: Some("h10".into()),
                    if_old_tags: None,
                },
            )
            .unwrap_err(),
            "Set tags if hash does not match".to_string()
        );

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                None,
                &mut file_tags,
                example_tags(),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: Some("h10".into()),
                    if_old_tags: None,
                },
            )
            .unwrap_err(),
            "Set tags if hash does not match".to_string()
        );
    }

    #[test]
    fn test_set_tags_conditions_if_old_tags() {
        let mut file_tags = example_tags();

        set_tags(
            Some(1),
            Some(2),
            Some("h1"),
            &mut file_tags,
            HashMap::from([("t1".into(), vec!["v1x".into(), "v2x".into()])]),
            &RemoteFileTagsSetConditions {
                if_size: None,
                if_modified: None,
                if_hash: None,
                if_old_tags: Some(HashMap::from([(
                    "t1".into(),
                    vec!["v1".into(), "v2".into()],
                )])),
            },
        )
        .unwrap();
        assert_eq!(
            file_tags,
            HashMap::from([
                ("t1".into(), vec!["v1x".into(), "v2x".into()]),
                ("t2".into(), vec!["v3".into()]),
            ])
        );
    }

    #[test]
    fn test_set_tags_conditions_if_old_tags_different_value() {
        let mut file_tags = example_tags();

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                HashMap::from([("t1".into(), vec!["v1x".into(), "v2x".into()])]),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: Some(HashMap::from([(
                        "t1".into(),
                        vec!["v1y".into(), "v2".into()],
                    )])),
                },
            )
            .unwrap_err(),
            "Set tags if old tags old value does not match the new value: t1".to_string()
        );
    }

    #[test]
    fn test_set_tags_conditions_if_old_tags_more_old_tags() {
        let mut file_tags = example_tags();

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                HashMap::from([("t1".into(), vec!["v1x".into()])]),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: Some(HashMap::from([("t1".into(), vec!["v1".into()],)])),
                },
            )
            .unwrap_err(),
            "Set tags if old tags number of old tags does not match the expected tags: 2 != 1"
                .to_string()
        );
    }

    #[test]
    fn test_set_tags_conditions_if_old_tags_less_old_tags() {
        let mut file_tags = example_tags();

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                HashMap::from([("t1".into(), vec!["v1x".into()])]),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: Some(HashMap::from([(
                        "t1".into(),
                        vec!["v1".into(), "v2".into(), "v3".into()],
                    )])),
                },
            )
            .unwrap_err(),
            "Set tags if old tags number of old tags does not match the expected tags: 2 != 3"
                .to_string()
        );
    }

    #[test]
    fn test_set_tags_conditions_if_old_tags_zero_old_tags() {
        let mut file_tags = HashMap::new();

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                HashMap::from([("t1".into(), vec!["v1x".into()])]),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: Some(HashMap::from([("t1".into(), vec!["v1".into()],)])),
                },
            )
            .unwrap_err(),
            "Set tags if old tags expected non-empty tags for key: t1".to_string()
        );
    }

    #[test]
    fn test_set_tags_conditions_if_old_tags_expected_zero_tags() {
        let mut file_tags = example_tags();

        assert_eq!(
            set_tags(
                Some(1),
                Some(2),
                Some("h1"),
                &mut file_tags,
                HashMap::from([("t1".into(), vec!["v1x".into()])]),
                &RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: Some(HashMap::from([("t1".into(), vec![],)])),
                },
            )
            .unwrap_err(),
            "Set tags if old tags expected empty tags for key: t1".to_string()
        );
    }
}
