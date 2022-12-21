pub fn split_name_ext<'a>(name: &'a str) -> (&'a str, Option<&'a str>) {
    name.rfind('.')
        .map(|idx| (&name[..idx], Some(&name[idx + 1..])))
        .unwrap_or((name, None))
}

pub fn name_to_ext<'a>(name: &'a str) -> Option<&'a str> {
    name.rfind('.').map(|idx| &name[idx + 1..])
}

pub fn join_name_ext(base_name: &str, ext: Option<&str>) -> String {
    ext.map(|ext| format!("{}.{}", base_name, ext))
        .unwrap_or_else(|| base_name.to_owned())
}

pub fn unused_name<F>(name: &str, exists: F) -> String
where
    F: Fn(&str) -> bool,
{
    if !exists(name) {
        return name.to_owned();
    }

    let (base_name, ext) = split_name_ext(name);

    let mut i = 1;

    loop {
        let new_name = join_name_ext(&format!("{} ({})", base_name, i), ext);

        i += 1;

        if !exists(&new_name) {
            return new_name;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::name_utils::{join_name_ext, name_to_ext, split_name_ext};

    use super::unused_name;

    #[test]
    fn test_split_name_ext() {
        assert_eq!(split_name_ext("test"), ("test", None));
        assert_eq!(split_name_ext("test.txt"), ("test", Some("txt")));
        assert_eq!(split_name_ext("test.tar.gz"), ("test.tar", Some("gz")));
    }

    #[test]
    fn test_name_to_ext() {
        assert_eq!(name_to_ext("test"), None);
        assert_eq!(name_to_ext("test.txt"), Some("txt"));
        assert_eq!(name_to_ext("test.tar.gz"), Some("gz"));
    }

    #[test]
    fn test_join_name_ext() {
        assert_eq!(join_name_ext("test", None), "test");
        assert_eq!(join_name_ext("test", Some("txt")), "test.txt");
        assert_eq!(join_name_ext("test.tar", Some("gz")), "test.tar.gz");
    }

    #[test]
    fn test_unused_name() {
        assert_eq!(unused_name("foo", |_| false), "foo");
        assert_eq!(unused_name("foo", |x| x == "foo"), "foo (1)");
        assert_eq!(
            unused_name("foo", |x| x == "foo" || x == "foo (1)"),
            "foo (2)"
        );

        assert_eq!(unused_name("foo.bar", |_| false), "foo.bar");
        assert_eq!(unused_name("foo.bar", |x| x == "foo.bar"), "foo (1).bar");
        assert_eq!(
            unused_name("foo.bar", |x| x == "foo.bar" || x == "foo (1).bar"),
            "foo (2).bar"
        );

        assert_eq!(unused_name("foo.bar.baz", |_| false), "foo.bar.baz");
        assert_eq!(
            unused_name("foo.bar.baz", |x| x == "foo.bar.baz"),
            "foo.bar (1).baz"
        );
        assert_eq!(
            unused_name("foo.bar.baz", |x| x == "foo.bar.baz"
                || x == "foo.bar (1).baz"),
            "foo.bar (2).baz"
        );
    }
}
