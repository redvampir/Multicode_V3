use git2::{BlameOptions, BranchType, Commit, DiffOptions, IndexAddOption, Repository};
use std::path::Path;
use tracing::error;

pub fn commit(message: &str) -> Result<(), git2::Error> {
    if message.trim().is_empty() {
        return Err(git2::Error::from_str("сообщение коммита не может быть пустым"));
    }

    let repo = Repository::discover(".")?;
    let mut index = repo.index()?;
    // Ограничиваем пути, которые могут быть добавлены в индекс, чтобы
    // случайно не закоммитить большие или несвязанные каталоги.
    let paths = ["backend", "frontend", "docs"];
    index.add_all(paths.iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Получаем подпись коммита из конфигурации репозитория. В некоторых
    // средах (например, в тестах или CI) у пользователя может не быть глобальной
    // конфигурации git. Ранее это приводило к ошибке `config value 'user.name' was not found`
    // и падению тестов в только что инициализированном репозитории. Вместо этого
    // пытаемся прочитать значения и используем заглушки по умолчанию, если они отсутствуют.
    let cfg = repo.config()?;
    let name = cfg
        .get_string("user.name")
        .unwrap_or_else(|_| "Unknown".to_string());
    let email = cfg
        .get_string("user.email")
        .unwrap_or_else(|_| "unknown@example.com".to_string());
    let sig = git2::Signature::now(&name, &email)?;
    // Определяем, есть ли в репозитории ссылка HEAD. В только что
    // инициализированном репозитории `head()` завершится ошибкой, что раньше
    // приводило к сбою `commit`. Вместо того чтобы передавать ошибку дальше,
    // обрабатываем начальный коммит явно: создаём его без обновления `HEAD`,
    // а затем устанавливаем `HEAD` на новый коммит.
    let head = repo.head();
    let parent = head.as_ref().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&Commit> = parent.iter().collect();

    if head.is_ok() {
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;
    } else {
        let oid = repo.commit(None, &sig, &sig, message, &tree, &parents)?;
        // При первом коммите создаём ветку "main", указывающую на новый
        // коммит, и переводим `HEAD` на неё. Это не позволяет оставить
        // репозиторий в состоянии отсоединённого `HEAD`, что может запутать
        // при дальнейшей работе.
        let branch = repo.branch("main", &repo.find_commit(oid)?, true)?;
        repo.set_head(branch.get().name().unwrap())?;
    }
    Ok(())
}

pub fn diff() -> Result<String, git2::Error> {
    const MAX_DIFF_LEN: usize = 100_000; // Ограничение вывода diff примерно 100 КБ

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
                error!("ошибка декодирования diff: {e}");
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

#[derive(Debug, serde::Serialize)]
pub struct BlameLine {
    pub line: usize,
    pub author: String,
    /// Секунды с начала эпохи Unix
    pub time: i64,
}

pub fn blame(path: &str) -> Result<Vec<BlameLine>, git2::Error> {
    let repo = Repository::discover(".")?;
    let mut opts = BlameOptions::new();
    let blame = repo.blame_file(Path::new(path), Some(&mut opts))?;
    let mut lines = Vec::new();
    for hunk in blame.iter() {
        let sig = hunk.final_signature();
        let author = sig.name().unwrap_or("Неизвестно").to_string();
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
