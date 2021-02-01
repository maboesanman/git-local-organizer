use core::fmt;
use std::{convert::TryFrom, error::Error, path::PathBuf};

use graphql_client::*;
use url::{ParseError, Url};

use crate::fork_resolution::{ForkResolution, RepoInfo};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "vendor/github/schema.graphql",
    query_path = "src/github/repo_parent.graphql",
    response_derives = "Debug"
)]
struct RepositoryParentInfo;

struct GithubRepository {
    host: String,
    owner: String,
    name: String,
}

impl TryFrom<GithubRepository> for RepoInfo {
    type Error = ParseError;

    fn try_from(value: GithubRepository) -> Result<Self, Self::Error> {
        let git_url = format!("https://{}/{}/{}.git", value.host, value.owner, value.name);
        let local_dir = PathBuf::new()
            .join(value.host)
            .join(value.owner.clone())
            .join(value.name);

        let repo_info = RepoInfo {
            git_url,
            local_dir,
            remote_name: value.owner,
        };

        Ok(repo_info)
    }
}

impl fmt::Display for GithubRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

impl GithubRepository {
    fn from_url(url: &Url) -> Result<Self, Box<dyn Error>> {
        let host = url.host_str().ok_or("not a github repo url")?.to_string();
        let path_segments: Vec<_> = url
            .path_segments()
            .ok_or("not a github repo url")?
            .collect();
        if path_segments.len() != 2 {
            Err("not a github repo url")?;
        }
        let owner = path_segments[0].to_string();
        let mut name = path_segments[1].to_string();
        if name.ends_with(".git") {
            name = name.split_at(name.len() - 4).0.to_string();
        }
        let new = Self { host, owner, name };

        Ok(new)
    }

    fn get_parent(&self, token: &str) -> Result<Option<Self>, Box<dyn Error>> {
        let q = RepositoryParentInfo::build_query(repository_parent_info::Variables {
            owner: self.owner.to_string(),
            name: self.name.to_string(),
        });

        let client = reqwest::blocking::Client::builder()
            .user_agent("graphql-rust/0.9.0")
            .build()?;

        let endpoint = format!("https://api.{}/graphql", self.host);
        let res = client.post(&endpoint).bearer_auth(token).json(&q).send()?;

        res.error_for_status_ref()?;

        let response_body: Response<repository_parent_info::ResponseData> = res.json()?;
        let response_data: repository_parent_info::ResponseData =
            response_body.data.ok_or("missing response data")?;
        let repo = response_data.repository.ok_or("repo not found")?;

        match repo.parent {
            Some(parent) => Ok(Some(Self {
                host: self.host.clone(),
                owner: parent.owner.login,
                name: parent.name,
            })),
            None => Ok(None),
        }
    }
}

pub fn resolve_forks(
    config: &crate::config::Config,
    url: &Url,
) -> Result<ForkResolution, Box<dyn Error>> {
    let mut base = GithubRepository::from_url(url)?;
    let api_token = config.get_github_api_token(&base.host)?;
    let mut children = Vec::new();

    loop {
        match base.get_parent(&api_token)? {
            Some(repo) => children.push(RepoInfo::try_from(std::mem::replace(&mut base, repo))?),
            None => {
                break Ok(ForkResolution {
                    base: RepoInfo::try_from(base)?,
                    children,
                })
            }
        }
    }
}
