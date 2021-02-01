use core::fmt;
use std::{error::Error, fs::File, io::BufReader};

use config::Config;
use directories_next::{ProjectDirs, UserDirs};
use fmt::Debug;
use git2::Repository;
use serde::*;
use structopt::StructOpt;

use url::Url;

mod config;
mod fork_resolution;
mod github;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    repo_url: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let proj_dirs = ProjectDirs::from("com", "maboesanman", "GitLocalOrganizer");
    let proj_dirs = proj_dirs.ok_or("can't read config directory")?;
    let user_dirs = UserDirs::new().ok_or("can't read user directory")?;
    let config_path = proj_dirs.config_dir().join("config.json");
    let config_file = File::open(config_path)?;
    let config_reader = BufReader::new(config_file);
    let config: Config = serde_json::from_reader(config_reader)?;
    let source_dir = user_dirs.home_dir().join(config.local_src_dir.clone());

    let args = Command::from_args();
    let repo_url = args.repo_url;
    let repo_url = Url::parse(&repo_url)?;
    let fork_resolution = github::resolve_forks(&config, &repo_url)?;
    let git_url = fork_resolution.base.git_url;
    let local_dir = source_dir.join(fork_resolution.base.local_dir);

    let repo = match Repository::clone_recurse(&git_url, &local_dir) {
        Ok(repo) => {
            println!(
                "cloning from \"{}\" into \"{}\"",
                git_url,
                local_dir.to_str().unwrap()
            );
            Ok(repo)
        }
        Err(e) => match e.code() {
            git2::ErrorCode::Exists => {
                println!("using existing repo at \"{}\"", local_dir.to_str().unwrap());
                Repository::open(&local_dir)
            }
            _ => Err(e),
        },
    }?;

    for remote in fork_resolution.children {
        match repo.remote(&remote.remote_name, &remote.git_url) {
            Ok(_) => {
                println!("added remote \"{}\"", &remote.remote_name);
                Ok(())
            }
            Err(e) => match e.code() {
                git2::ErrorCode::Exists => {
                    println!("remote \"{}\" exists; skipping", &remote.remote_name);
                    Ok(())
                }
                _ => Err(e),
            },
        }?;
        let remote_link_path = source_dir.join(remote.local_dir.clone());
        let mut remote_link_parent_path = remote_link_path.clone();
        remote_link_parent_path.pop();
        std::fs::create_dir_all(&remote_link_parent_path)?;
        match std::os::unix::fs::symlink(&local_dir, &remote_link_path) {
            Ok(()) => println!(
                "symlink created for remote \"{}\"",
                remote_link_path.to_str().unwrap()
            ),
            Err(_) => println!(
                "could not create symlink for remote \"{}\"; skipping",
                remote_link_path.to_str().unwrap()
            ),
        }
    }

    println!("Done 😎");
    Ok(())
}
