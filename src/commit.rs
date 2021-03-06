use std::process::Command;

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use chrono_english::{parse_date_string, DateError, Dialect};
use git2::{PushOptions, Repository};
use git2_credentials::CredentialHandler;

use crate::config::ConfigFile;
use crate::data::CommitLog;

pub struct Committer<'a> {
    options: PushOptions<'a>,
    repo: &'a Repository,
    config: &'a mut ConfigFile,
}

impl<'a> Committer<'a> {
    pub fn new(config: &'a mut ConfigFile, repo: &'a Repository) -> Self {
        Committer {
            repo,
            config,
            options: Self::get_push_options(),
        }
    }

    pub fn commit(&mut self, message: String, date: Option<&str>) -> Result<(), Error> {
        let sig = self.repo.signature().unwrap();
        let head = self.repo.head().unwrap();
        let active_branch = head.shorthand().unwrap();
        let date_time = self
            .get_datetime(date)
            .expect("Passed date time is not parsable");

        let tree_id = self.repo.index().unwrap().write_tree().unwrap();
        let tree = self.repo.find_tree(tree_id).unwrap();

        let parent_id = self.repo.head().ok().and_then(|h| h.target()).unwrap();
        let parent = self.repo.find_commit(parent_id).unwrap();

        //Commit and tag the commit
        let commit_id = self
            .repo
            .commit(Some("HEAD"), &sig, &sig, &message, &tree, &[&parent])
            .unwrap();
        self.config.add_log(CommitLog::new(
            commit_id.to_string(),
            active_branch.to_string(),
            date_time,
        ))?;
        Ok(())
    }

    pub fn push(&mut self, branch: &str) -> Result<(), Error>{
        //Get commit till which can be pushed
        let commit_id = self.config.get_commit(Utc::now());

        let mut remote = self.repo.find_remote("origin").unwrap();
        let origin = format!("refs/heads/{}", branch);

        if commit_id.is_some() {
            let commit_id = commit_id.unwrap();
            let exit_code = Command::new("git")
                .arg("push")
                .arg("origin")
                .arg(format!("{}:{}", &commit_id, branch))
                .spawn()
                .expect("Push failed")
                .wait()
                .unwrap();
            if exit_code.code().unwrap() == 0 {
                self.config.clear_logs(commit_id.as_str())?;
            } else {
                println!("Push failed");
            }
        } else if !self.config.has_logs() {
             remote.push(&[&origin], Some(&mut self.options))?;
             println!("Push is successful");
        } else {
            println!("Nothing to push");
        }
        Ok(())
    }

    pub fn print_logs(&self) {
        self.config.print_logs();
    }

    pub fn print_next_commit(&self) {
        self.config.print_next_commit();
    }

    fn get_push_options() -> PushOptions<'a> {
        let mut opts = git2::PushOptions::new();
        let mut cb = git2::RemoteCallbacks::new();
        let git_config = git2::Config::open_default().unwrap();
        let mut ch = CredentialHandler::new(git_config);
        cb.credentials(move |url, username, allowed| {
            ch.try_next_credential(url, username, allowed)
        });
        opts.remote_callbacks(cb);

        opts
    }

    fn get_datetime(&self, expr: Option<&str>) -> Result<DateTime<Utc>, DateError> {
        match expr {
            Some(str) => parse_date_string(str, Utc::now(), Dialect::Uk),
            None => Ok(Utc::now()),
        }
    }
}
