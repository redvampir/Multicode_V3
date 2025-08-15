use git2::{BlameOptions, BranchType, Commit, DiffOptions, IndexAddOption, Repository};
use std::path::Path;
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

    // Obtain commit signature from repository configuration.  In some
    // environments (for example in tests or CI) the user may not have global
    // git configuration set up.  Previously we bubbled up the `config value
    // 'user.name' was not found` error which caused the commit tests to fail
    // on a freshly initialised repository.  Instead, attempt to read the values
    // and fall back to placeholder defaults when they are missing.
    let cfg = repo.config()?;
    let name = cfg
        .get_string("user.name")
        .unwrap_or_else(|_| "Unknown".to_string());
    let email = cfg
        .get_string("user.email")
        .unwrap_or_else(|_| "unknown@example.com".to_string());
    let sig = git2::Signature::now(&name, &email)?;
    // Determine whether the repository already has a HEAD reference. In a
    // freshly initialised repository `head()` will fail which previously caused
    // `commit` to error. Instead of bubbling this error up we handle the
    // initial commit case explicitly by committing without updating `HEAD` and
    // then setting `HEAD` to point to the new commit.
    let head = repo.head();
    let parent = head.as_ref().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&Commit> = parent.iter().collect();

    if head.is_ok() {
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    } else {
        let oid = repo.commit(None, &sig, &sig, message, &tree, &parents)?;
        repo.set_head_detached(oid)?;
    }
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

#[derive(serde::Serialize)]
pub struct BlameLine {
    pub line: usize,
    pub author: String,
    /// Seconds since Unix epoch
    pub time: i64,
}

pub fn blame(path: &str) -> Result<Vec<BlameLine>, git2::Error> {
    let repo = Repository::discover(".")?;
    let mut opts = BlameOptions::new();
    let blame = repo.blame_file(Path::new(path), Some(&mut opts))?;
    let mut lines = Vec::new();
    for hunk in blame.iter() {
        let sig = hunk.final_signature();
        let author = sig.name().unwrap_or("Unknown").to_string();
        let time = sig.when().seconds();
        let mut line_no = hunk.final_start_line();
        for _ in 0..hunk.lines_in_hunk() {
            lines.push(BlameLine {
                line: line_no as usize,
                author: author.clone(),
                time,
            });
            line_no += 1;
        }
    }
    Ok(lines)
}
