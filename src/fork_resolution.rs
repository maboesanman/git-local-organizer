use std::path::PathBuf;

pub struct ForkResolution {
    pub base: RepoInfo,
    pub children: Vec<RepoInfo>,
}

pub struct RepoInfo {
    pub git_url: String,
    pub local_dir: PathBuf,
    pub remote_name: String,
}
