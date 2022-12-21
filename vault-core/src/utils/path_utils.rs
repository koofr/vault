pub fn path_to_name<'a>(path: &'a str) -> Option<&'a str> {
    match path {
        "/" => None,
        _ => {
            let idx = path.rfind('/').unwrap();

            Some(&path[idx + 1..])
        }
    }
}

pub fn join_path_name(path: &str, name: &str) -> String {
    match path {
        "/" => path.to_owned() + name,
        path => path.to_owned() + "/" + name,
    }
}

pub fn join_paths(parent_path: &str, path: &str) -> String {
    match (parent_path, path) {
        (_, "/") => parent_path.to_owned(),
        ("/", _) => path.to_owned(),
        (_, _) => parent_path.to_owned() + path,
    }
}

pub fn parent_path<'a>(path: &'a str) -> Option<&'a str> {
    if path == "/" {
        None
    } else {
        let idx = path.rfind('/').unwrap();

        if idx == 0 {
            Some("/")
        } else {
            Some(&path[..idx])
        }
    }
}

pub fn split_parent_name<'a>(path: &'a str) -> Option<(&'a str, &'a str)> {
    if path == "/" {
        None
    } else {
        let idx = path.rfind('/').unwrap();

        if idx == 0 {
            Some(("/", &path[idx + 1..]))
        } else {
            Some((&path[..idx], &path[idx + 1..]))
        }
    }
}

/// /foo/bar => [/, /foo, /foo/bar]
pub fn paths_chain(path: &str) -> Vec<String> {
    let mut chain = Vec::new();

    let mut path = path.to_owned();

    loop {
        match parent_path(&path) {
            Some(parent) => {
                chain.push(path.clone());

                path = parent.to_owned();
            }
            None => {
                break;
            }
        }
    }

    chain.push(String::from("/"));

    chain.reverse();

    chain
}

#[cfg(test)]
mod tests {
    use super::{join_paths, parent_path, path_to_name, paths_chain, split_parent_name};

    #[test]
    fn test_path_to_name() {
        assert_eq!(path_to_name("/"), None);
        assert_eq!(path_to_name("/foo"), Some("foo"));
        assert_eq!(path_to_name("/foo/bar"), Some("bar"));
    }

    #[test]
    fn test_join_paths() {
        assert_eq!(join_paths("/", "/"), "/");
        assert_eq!(join_paths("/foo", "/"), "/foo");
        assert_eq!(join_paths("/", "/foo"), "/foo");
        assert_eq!(join_paths("/foo", "/bar"), "/foo/bar");
    }

    #[test]
    fn test_parent_path() {
        assert_eq!(parent_path("/"), None);
        assert_eq!(parent_path("/foo"), Some("/"));
        assert_eq!(parent_path("/foo/bar"), Some("/foo"));
    }

    #[test]
    fn test_split_parent_name() {
        assert_eq!(split_parent_name("/"), None);
        assert_eq!(split_parent_name("/foo"), Some(("/", "foo")));
        assert_eq!(split_parent_name("/foo/bar"), Some(("/foo", "bar")));
        assert_eq!(split_parent_name("/foo/bar/baz"), Some(("/foo/bar", "baz")));
    }

    #[test]
    fn test_paths_chain() {
        assert_eq!(paths_chain("/"), vec![String::from("/")]);
        assert_eq!(
            paths_chain("/foo"),
            vec![String::from("/"), String::from("/foo")]
        );
        assert_eq!(
            paths_chain("/foo/bar"),
            vec![
                String::from("/"),
                String::from("/foo"),
                String::from("/foo/bar")
            ]
        );
    }
}
