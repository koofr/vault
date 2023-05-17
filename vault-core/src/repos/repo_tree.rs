#[derive(Clone)]
struct RepoTreeNode {
    repo_id: Option<String>,
    /// Vec lookups are faster than HashMap for small amounts of children
    children: Vec<(String, usize)>,
}

/// RepoTree is a trie (prefix tree) for quickly resolving remote paths to
/// repos. RepoTree contains repos for a single mount.
#[derive(Clone)]
pub struct RepoTree {
    nodes: Vec<RepoTreeNode>,
}

impl RepoTree {
    pub fn new() -> Self {
        Self {
            nodes: vec![RepoTreeNode {
                repo_id: None,
                children: vec![],
            }],
        }
    }

    /// Get returns all (repo_id, repo_path) pairs that for a path.
    pub fn get<'a, 'b>(&'a self, path: &'b str) -> Vec<(&'a str, &'b str)> {
        assert_valid_path(path);

        let mut current_path = path;
        let mut node_idx: usize = 0;

        let mut pairs: Vec<(&'a str, &'b str)> = Vec::new();

        loop {
            if let Some(repo_id) = &self.nodes[node_idx].repo_id {
                pairs.push((repo_id, current_path));
            }

            match current_path {
                "/" => break,
                _ => {
                    let (key, tail) = path_to_key_tail(current_path);

                    match self.find_child(node_idx, &key) {
                        Some(idx) => node_idx = idx,
                        None => break,
                    }

                    current_path = tail;
                }
            }
        }

        pairs
    }

    pub fn set(&mut self, path: &str, repo_id: String) {
        assert_valid_path(path);

        let mut current_path = path;
        let mut node_idx: usize = 0;

        loop {
            match current_path {
                "/" => break,
                _ => {
                    let (key, tail) = path_to_key_tail(current_path);

                    node_idx = self.find_or_add_child(node_idx, key);

                    current_path = tail;
                }
            }
        }

        self.nodes[node_idx].repo_id = Some(repo_id);
    }

    pub fn remove(&mut self, path: &str) -> Option<String> {
        assert_valid_path(path);

        let mut current_path = path;
        let mut node_idx: usize = 0;

        loop {
            match current_path {
                "/" => break,
                _ => {
                    let (key, tail) = path_to_key_tail(current_path);

                    match self.find_child(node_idx, &key) {
                        Some(idx) => node_idx = idx,
                        None => {
                            return None;
                        }
                    }

                    current_path = tail;
                }
            }
        }

        self.nodes[node_idx].repo_id.take()
    }

    fn find_child(&self, node_idx: usize, key: &str) -> Option<usize> {
        self.nodes[node_idx]
            .children
            .iter()
            .find(|child| child.0 == key)
            .map(|x| x.1)
    }

    fn add_child(&mut self, node_idx: usize, key: String) -> usize {
        let new_idx = self.add_node();

        self.nodes[node_idx].children.push((key, new_idx));

        new_idx
    }

    fn find_or_add_child(&mut self, node_idx: usize, key: String) -> usize {
        self.find_child(node_idx, &key)
            .unwrap_or(self.add_child(node_idx, key))
    }

    fn add_node(&mut self) -> usize {
        self.nodes.push(RepoTreeNode {
            repo_id: None,
            children: vec![],
        });

        self.nodes.len() - 1
    }
}

fn assert_valid_path(path: &str) {
    assert!(path.starts_with('/'), "path does not have / prefix");
    assert!(!path.contains("//"), "path has //");
}

fn path_to_key_tail<'a>(path: &'a str) -> (String, &'a str) {
    match path[1..].find('/') {
        Some(idx) => (path[1..idx + 1].to_lowercase(), &path[idx + 1..]),
        None => (path[1..].to_lowercase(), &path[0..1]),
    }
}

#[cfg(test)]
mod tests {
    use super::{path_to_key_tail, RepoTree};

    #[test]
    fn test_repo_tree() {
        let mut t = RepoTree::new();

        assert_eq!(t.get("/"), vec![]);
        assert_eq!(t.get("/d1"), vec![]);
        assert_eq!(t.get("/D1"), vec![]);
        assert_eq!(t.get("/D2"), vec![]);

        t.set("/", String::from("r1"));
        assert_eq!(t.get("/"), vec![("r1", "/")]);
        assert_eq!(t.get("/d1"), vec![("r1", "/d1")]);
        assert_eq!(t.get("/D1"), vec![("r1", "/D1")]);
        assert_eq!(t.get("/D2"), vec![("r1", "/D2")]);

        t.set("/D1", String::from("r2"));
        assert_eq!(t.get("/d1"), vec![("r1", "/d1"), ("r2", "/")]);
        assert_eq!(t.get("/D1"), vec![("r1", "/D1"), ("r2", "/")]);
        assert_eq!(t.get("/d1/d11"), vec![("r1", "/d1/d11"), ("r2", "/d11")]);
        assert_eq!(t.get("/D1/D11"), vec![("r1", "/D1/D11"), ("r2", "/D11")]);
        assert_eq!(t.get("/D2"), vec![("r1", "/D2")]);

        assert_eq!(t.remove("/"), Some(String::from("r1")));
        assert_eq!(t.get("/"), vec![]);
        assert_eq!(t.get("/d1"), vec![("r2", "/")]);
        assert_eq!(t.get("/D1"), vec![("r2", "/")]);
        assert_eq!(t.get("/d1/d11"), vec![("r2", "/d11")]);
        assert_eq!(t.get("/D1/D11"), vec![("r2", "/D11")]);
        assert_eq!(t.get("/D2"), vec![]);

        assert_eq!(t.remove("/D1"), Some(String::from("r2")));
        assert_eq!(t.get("/"), vec![]);
        assert_eq!(t.get("/d1"), vec![]);
        assert_eq!(t.get("/D1"), vec![]);
        assert_eq!(t.get("/d1/d11"), vec![]);
        assert_eq!(t.get("/D1/D11"), vec![]);
        assert_eq!(t.get("/D2"), vec![]);
    }

    #[test]
    fn test_repo_tree_child() {
        let mut t = RepoTree::new();

        t.set("/path/to/r1", String::from("r1"));
        assert_eq!(t.get("/"), vec![]);
        assert_eq!(t.get("/path"), vec![]);
        assert_eq!(t.get("/path/to"), vec![]);
        assert_eq!(t.get("/path/to/r1"), vec![("r1", "/")]);
        assert_eq!(t.get("/Path/to/r1/d1"), vec![("r1", "/d1")]);
        assert_eq!(t.get("/Path/to/r1/D1"), vec![("r1", "/D1")]);
    }

    #[test]
    fn test_path_to_key_tail() {
        assert_eq!(path_to_key_tail("/FOO"), (String::from("foo"), "/"));
        assert_eq!(path_to_key_tail("/FOO/Bar"), (String::from("foo"), "/Bar"));
    }
}
