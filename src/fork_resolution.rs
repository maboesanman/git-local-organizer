use std::path::PathBuf;

#[derive(Debug)]
pub struct ForkResolution {
    pub base: RepoInfo,
    pub children: Vec<RepoInfo>,
}

#[derive(Debug)]
pub struct RepoInfo {
    pub git_url: String,
    pub local_dir: PathBuf,
    pub remote_name: String,
}
