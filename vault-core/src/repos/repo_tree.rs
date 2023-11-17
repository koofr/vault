use crate::types::{EncryptedPath, RemotePath, RepoId};

#[derive(Debug, Clone)]
struct RepoTreeNode {
    repo_id: Option<RepoId>,
    /// Vec lookups are faster than HashMap for small amounts of children
    children: Vec<(String, usize)>,
}

/// RepoTree is a trie (prefix tree) for quickly resolving remote paths to
/// repos. RepoTree contains repos for a single mount.
#[derive(Debug, Clone)]
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
    pub fn get<'a>(&'a self, path: &RemotePath) -> Vec<(&'a RepoId, EncryptedPath)> {
        assert_valid_path(path);

        let mut current_path = path.0.as_str();
        let mut node_idx: usize = 0;

        let mut pairs: Vec<(&'a RepoId, EncryptedPath)> = Vec::new();

        loop {
            if let Some(repo_id) = &self.nodes[node_idx].repo_id {
                pairs.push((repo_id, EncryptedPath(current_path.to_owned())));
            }

            if current_path == "/" {
                break;
            }

            let (key, tail) = path_to_key_tail(current_path);

            match self.find_child(node_idx, &key) {
                Some(idx) => node_idx = idx,
                None => break,
            }

            current_path = tail;
        }

        pairs
    }

    pub fn set(&mut self, path: &RemotePath, repo_id: RepoId) {
        assert_valid_path(path);

        let mut current_path = path.0.as_str();
        let mut node_idx: usize = 0;

        while current_path != "/" {
            let (key, tail) = path_to_key_tail(current_path);

            node_idx = self.find_or_add_child(node_idx, key);

            current_path = tail;
        }

        self.nodes[node_idx].repo_id = Some(repo_id);
    }

    pub fn remove(&mut self, path: &RemotePath) -> Option<RepoId> {
        assert_valid_path(path);

        let mut current_path = path.0.as_str();
        let mut node_idx: usize = 0;

        while current_path != "/" {
            let (key, tail) = path_to_key_tail(current_path);

            match self.find_child(node_idx, &key) {
                Some(idx) => node_idx = idx,
                None => {
                    return None;
                }
            }

            current_path = tail;
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

fn assert_valid_path(path: &RemotePath) {
    assert!(path.0.starts_with('/'), "path does not have / prefix");
    assert!(!path.0.contains("//"), "path has //");
}

fn path_to_key_tail<'a>(path: &'a str) -> (String, &'a str) {
    match path[1..].find('/') {
        Some(idx) => (path[1..idx + 1].to_lowercase(), &path[idx + 1..]),
        None => (path[1..].to_lowercase(), &path[0..1]),
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{EncryptedPath, RemotePath, RepoId};

    use super::{path_to_key_tail, RepoTree};

    #[test]
    fn test_repo_tree() {
        let mut t = RepoTree::new();

        assert_eq!(t.get(&RemotePath("/".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/d1".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/D1".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/D2".into())), vec![]);

        t.set(&RemotePath("/".into()), RepoId("r1".into()));
        assert_eq!(
            t.get(&RemotePath("/".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/d1".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/d1".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/D1".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/D1".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/D2".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/D2".into()))]
        );

        t.set(&RemotePath("/D1".into()), RepoId("r2".into()));
        assert_eq!(
            t.get(&RemotePath("/d1".into())),
            vec![
                (&RepoId("r1".into()), EncryptedPath("/d1".into())),
                (&RepoId("r2".into()), EncryptedPath("/".into()))
            ]
        );
        assert_eq!(
            t.get(&RemotePath("/D1".into())),
            vec![
                (&RepoId("r1".into()), EncryptedPath("/D1".into())),
                (&RepoId("r2".into()), EncryptedPath("/".into()))
            ]
        );
        assert_eq!(
            t.get(&RemotePath("/d1/d11".into())),
            vec![
                (&RepoId("r1".into()), EncryptedPath("/d1/d11".into())),
                (&RepoId("r2".into()), EncryptedPath("/d11".into()))
            ]
        );
        assert_eq!(
            t.get(&RemotePath("/D1/D11".into())),
            vec![
                (&RepoId("r1".into()), EncryptedPath("/D1/D11".into())),
                (&RepoId("r2".into()), EncryptedPath("/D11".into()))
            ]
        );
        assert_eq!(
            t.get(&RemotePath("/D2".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/D2".into()))]
        );

        assert_eq!(t.remove(&RemotePath("/".into())), Some(RepoId("r1".into())));
        assert_eq!(t.get(&RemotePath("/".into())), vec![]);
        assert_eq!(
            t.get(&RemotePath("/d1".into())),
            vec![(&RepoId("r2".into()), EncryptedPath("/".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/D1".into())),
            vec![(&RepoId("r2".into()), EncryptedPath("/".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/d1/d11".into())),
            vec![(&RepoId("r2".into()), EncryptedPath("/d11".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/D1/D11".into())),
            vec![(&RepoId("r2".into()), EncryptedPath("/D11".into()))]
        );
        assert_eq!(t.get(&RemotePath("/D2".into())), vec![]);

        assert_eq!(
            t.remove(&RemotePath("/D1".into())),
            Some(RepoId("r2".into()))
        );
        assert_eq!(t.get(&RemotePath("/".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/d1".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/D1".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/d1/d11".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/D1/D11".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/D2".into())), vec![]);
    }

    #[test]
    fn test_repo_tree_child() {
        let mut t = RepoTree::new();

        t.set(&RemotePath("/path/to/r1".into()), RepoId("r1".into()));
        assert_eq!(t.get(&RemotePath("/".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/path".into())), vec![]);
        assert_eq!(t.get(&RemotePath("/path/to".into())), vec![]);
        assert_eq!(
            t.get(&RemotePath("/path/to/r1".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/Path/to/r1/d1".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/d1".into()))]
        );
        assert_eq!(
            t.get(&RemotePath("/Path/to/r1/D1".into())),
            vec![(&RepoId("r1".into()), EncryptedPath("/D1".into()))]
        );
    }

    #[test]
    fn test_path_to_key_tail() {
        assert_eq!(path_to_key_tail("/FOO"), (String::from("foo"), "/"));
        assert_eq!(path_to_key_tail("/FOO/Bar"), (String::from("foo"), "/Bar"));
    }
}
