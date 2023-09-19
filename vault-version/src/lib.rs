#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    pub git_revision: Option<String>,
    pub git_revision_url: Option<String>,
    pub git_release: Option<String>,
    pub git_release_url: Option<String>,
}

impl Version {
    pub fn new() -> Self {
        let git_revision = option_env!("GIT_REVISION")
            .filter(|x| !x.is_empty())
            .map(str::to_string);
        let git_revision_url = git_revision
            .as_ref()
            .map(|x| format!("https://github.com/koofr/vault/commit/{}", x));
        let git_release = option_env!("GIT_RELEASE")
            .filter(|x| !x.is_empty())
            .map(str::to_string);
        let git_release_url = git_release
            .as_ref()
            .map(|x| format!("https://github.com/koofr/vault/releases/tag/{}", x));

        Self {
            git_revision,
            git_revision_url,
            git_release,
            git_release_url,
        }
    }
}
