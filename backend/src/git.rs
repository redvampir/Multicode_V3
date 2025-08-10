use git2::{Repository, IndexAddOption, DiffOptions, BranchType, Commit};

pub fn commit(message: &str) -> Result<(), git2::Error> {
    let repo = Repository::discover(".")?;
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let sig = repo.signature()?;
    let head = repo.head().ok();
    let parent = head.and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&Commit> = parent.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(())
}

pub fn diff() -> Result<String, git2::Error> {
    let repo = Repository::discover(".")?;
    let mut opts = DiffOptions::new();
    opts.include_untracked(true);
    let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;
    let mut out = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        out.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
        true
    })?;
    Ok(out)
}

pub fn branches() -> Result<Vec<String>, git2::Error> {
    let repo = Repository::discover(".")?;
    let mut names = Vec::new();
    for branch in repo.branches(Some(BranchType::Local))? {
        let (b, _) = branch?;
        if let Some(name) = b.name()? {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

pub fn log() -> Result<Vec<String>, git2::Error> {
    let repo = Repository::discover(".")?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let mut entries = Vec::new();
    for oid_res in revwalk.take(20) {
        let oid = oid_res?;
        let commit = repo.find_commit(oid)?;
        let msg = commit.summary().unwrap_or("").to_string();
        entries.push(format!("{} {}", &oid.to_string()[..7], msg));
    }
    Ok(entries)
}
