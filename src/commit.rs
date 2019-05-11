use git2::{BranchType, Repository};

use crate::config::ConfigFile;
use crate::data;
use crate::data::CommitLog;

pub fn commit(message: String, date: String, config: &mut ConfigFile, repo: &Repository) {
    let sig = repo.signature().unwrap();
    let mut index = repo.index().unwrap();

    println!("{}", &sig.email().unwrap());

    //let head = repo.head().unwrap().peel_to_commit().unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let parent_id = repo.head().ok().and_then(|h| h.target()).unwrap();
    let parent = repo.find_commit(parent_id).unwrap();

    let commit_id = repo.commit(Some("HEAD"), &sig,
                                &sig, &message, &tree, &[&parent]).unwrap();

    config.add_log(CommitLog::new(commit_id.to_string()));
}

pub fn push(branch: &str, config: &mut ConfigFile, repo: &Repository) {
    let mut remote = repo.find_remote("origin").unwrap();
    let mut opts = git2::PushOptions::new();
    let origin = format!("refs/remotes/origin/{}", active_branch).as_str();

    match remote.push(&[origin], Some(&mut opts)) {
        Err(e) => println!("Push to remote failed"),
        Ok(_) => println!("Push successful")
    }
}
