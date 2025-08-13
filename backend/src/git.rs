use git2::{BranchType, Commit, DiffOptions, IndexAddOption, Repository};
use tracing::error;

pub fn commit(message: &str) -> Result<(), git2::Error> {
    if message.trim().is_empty() {
        return Err(git2::Error::from_str("commit message cannot be empty"));
    }

    let repo = Repository::discover(".")?;
    let mut index = repo.index()?;
    // Limit the paths that can be added to the index to avoid accidentally
    // committing large or unrelated directories.
    let paths = ["backend", "frontend", "docs"];
    index.add_all(paths.iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Obtain commit signature from repository configuration to avoid relying on
    // global git config which may be absent in test environments.
    let cfg = repo.config()?;
    let name = cfg.get_string("user.name")?;
    let email = cfg.get_string("user.email")?;
    let sig = git2::Signature::now(&name, &email)?;
    let head = repo.head().ok();
    let parent = head.and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&Commit> = parent.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    Ok(())
}

pub fn diff() -> Result<String, git2::Error> {
    const MAX_DIFF_LEN: usize = 100_000; // Limit diff output to ~100 KB

    let repo = Repository::discover(".")?;
    let mut opts = DiffOptions::new();
    opts.include_untracked(true);
    let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;
    let mut out = String::new();

    diff.print(
        git2::DiffFormat::Patch,
        |_delta, _hunk, line| match std::str::from_utf8(line.content()) {
            Ok(s) => {
                if out.len() + s.len() <= MAX_DIFF_LEN {
                    out.push_str(s);
                    true
                } else {
                    let remaining = MAX_DIFF_LEN.saturating_sub(out.len());
                    out.push_str(&s[..remaining]);
                    false
                }
            }
            Err(e) => {
                error!("diff decode error: {e}");
                true
            }
        },
    )?;

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
