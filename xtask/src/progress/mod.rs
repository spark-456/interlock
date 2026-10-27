//! `cargo xtask progress`: the data files behind the progress tracker, read from the board, git, GitHub and the tree.
//!
//! Every file is a JSON array of row objects. GitHub data needs a signed-in `gh`; without it those files are empty.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

mod tree;
use tree::{modules, nodes, screens};

/// Writes one `<dataset>.json` per tracker dataset into `out`.
pub fn write(root: &Path, out: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(out)?;
    let board = fs::read_to_string(root.join("docs/WORKBOARD.md"))?;
    let mapping = fs::read_to_string(root.join("design/interlock/MAPPING.md"))?;
    let tasks = tasks(&board);
    let datasets = [
        ("snapshot", snapshot(root)),
        ("owner", table(&board, "## Waiting on the owner")),
        ("blockers", table(&board, "## Blockers and known issues")),
        ("prs", prs()),
        ("issues", issues()),
        ("ci", ci()),
        ("commits", commits(root)),
        ("modules", modules(root)),
        ("nodes", nodes(root)?),
        ("screens", screens(root, &mapping, &tasks)),
        ("tasks", Value::Array(tasks)),
    ];
    for (name, rows) in datasets {
        let n = rows.as_array().map_or(0, Vec::len);
        fs::write(out.join(format!("{name}.json")), serde_json::to_string_pretty(&rows)? + "\n")?;
        eprintln!("{name}.json: {n} rows");
    }
    Ok(())
}

/// When and from which commit the snapshot was taken.
fn snapshot(root: &Path) -> Value {
    let head = git(root, &["log", "-1", "--format=%h%x09%s"]);
    let (commit, subject) = head.trim().split_once('\t').unwrap_or(("", ""));
    json!([{ "generatedAt": now_iso(), "commit": commit, "subject": subject, "branch": git(root, &["branch", "--show-current"]).trim() }])
}

/// The board's index table, one row per task, with its area and whether its hard deps are done.
fn tasks(board: &str) -> Vec<Value> {
    let rows = table(board, "## Index");
    let rows = rows.as_array().cloned().unwrap_or_default();
    let done: Vec<String> =
        rows.iter().filter(|r| r["status"] == "done").filter_map(|r| r["id"].as_str().map(String::from)).collect();
    rows.into_iter()
        .map(|r| {
            let id = r["id"].as_str().unwrap_or_default().to_string();
            let deps: Vec<String> = r["hardDeps"]
                .as_str()
                .unwrap_or_default()
                .split(',')
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty() && d != "—")
                .collect();
            let deps_done = deps.iter().all(|d| done.contains(d));
            json!({
                "order": r["#"].as_str().and_then(|n| n.parse::<u32>().ok()),
                "id": id,
                "title": r["task"],
                "area": area(&id),
                "prio": r["prio"],
                "deps": deps.join(", "),
                "depsDone": deps_done,
                "status": r["status"],
                "owner": r["owner"],
                "branch": r["branch"],
                "updated": board_time(r["updatedUtc"].as_str().unwrap_or_default()),
                "notes": r["prNotes"],
            })
        })
        .collect()
}

/// A task's area, from its ID prefix.
fn area(id: &str) -> &'static str {
    match id.chars().next() {
        Some('T') => "Foundation",
        Some('B') => "Backend",
        Some('W') => "Frontend",
        Some('A') => "Architect",
        Some('K') => "Knowledge",
        Some('P') => "Publish",
        Some('C') => "Connectors",
        Some('M') => "Media",
        Some('N') => "Nodes",
        Some('R') => "Release",
        _ => "Other",
    }
}

/// The first markdown table after `heading`, as rows keyed by camelCased header names.
fn table(doc: &str, heading: &str) -> Value {
    let Some(start) = doc.find(heading) else { return json!([]) };
    let lines = doc[start..].lines().skip(1).skip_while(|l| !l.starts_with('|')).take_while(|l| l.starts_with('|'));
    let mut rows = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    for (i, line) in lines.enumerate() {
        let cells: Vec<String> = line.trim().trim_matches('|').split('|').map(|c| plain(c.trim())).collect();
        match i {
            0 => keys = cells.iter().map(|c| camel(c)).collect(),
            1 => {}
            _ if cells.iter().all(String::is_empty) => {}
            _ => rows.push(Value::Object(keys.iter().cloned().zip(cells.into_iter().map(Value::String)).collect())),
        }
    }
    Value::Array(rows)
}

/// Markdown cell text without emphasis or code marks.
fn plain(cell: &str) -> String {
    cell.replace("**", "").replace('`', "")
}

/// `"Updated (UTC)"` → `"updatedUtc"`, `"#"` stays `"#"`.
fn camel(header: &str) -> String {
    let words: Vec<String> = header
        .split(|c: char| !c.is_alphanumeric() && c != '#')
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    let mut out = String::new();
    for (i, w) in words.iter().enumerate() {
        if i == 0 {
            out.push_str(w);
        } else {
            let mut c = w.chars();
            out.extend(c.next().map(|f| f.to_ascii_uppercase()));
            out.push_str(c.as_str());
        }
    }
    out
}

/// `"2026-10-27 05:15"` → `"2026-10-27T05:15:00Z"`; anything else → null.
fn board_time(s: &str) -> Value {
    let s = s.trim();
    if s.len() == 16 && s.as_bytes()[10] == b' ' {
        json!(format!("{}T{}:00Z", &s[..10], &s[11..]))
    } else {
        Value::Null
    }
}

/// Pull requests (all states), with the task ID taken from a `[ID]` title prefix.
fn prs() -> Value {
    let fields =
        "number,title,state,isDraft,headRefName,createdAt,mergedAt,closedAt,additions,deletions,changedFiles,url";
    let mut rows = gh(&["pr", "list", "--state", "all", "--limit", "200", "--json", fields]);
    for r in rows.iter_mut() {
        let title = r["title"].as_str().unwrap_or_default();
        let task = title.strip_prefix('[').and_then(|t| t.split_once(']')).map(|(id, _)| id.to_string());
        r["task"] = json!(task);
    }
    Value::Array(rows)
}

/// GitHub issues (all states), labels flattened to text.
fn issues() -> Value {
    let mut rows = gh(&[
        "issue",
        "list",
        "--state",
        "all",
        "--limit",
        "200",
        "--json",
        "number,title,state,createdAt,closedAt,labels,url",
    ]);
    for r in rows.iter_mut() {
        let labels: Vec<&str> =
            r["labels"].as_array().map(|l| l.iter().filter_map(|x| x["name"].as_str()).collect()).unwrap_or_default();
        r["labels"] = json!(labels.join(", "));
    }
    Value::Array(rows)
}

/// The latest CI runs.
fn ci() -> Value {
    let fields = "databaseId,displayTitle,headBranch,event,status,conclusion,createdAt,updatedAt,url";
    Value::Array(gh(&["run", "list", "--limit", "30", "--json", fields]))
}

/// Every commit on HEAD, oldest first, with line counts and the running total of lines.
fn commits(root: &Path) -> Value {
    let log = git(root, &["log", "--reverse", "--numstat", "--format=@@%h%x09%cI%x09%an%x09%s"]);
    let mut rows: Vec<Map<String, Value>> = Vec::new();
    let mut total: i64 = 0;
    for line in log.lines() {
        if let Some(head) = line.strip_prefix("@@") {
            let p: Vec<&str> = head.splitn(4, '\t').collect();
            if p.len() == 4 {
                let row = json!({"hash": p[0], "date": p[1], "author": p[2], "subject": p[3],
                    "task": task_of(p[3]), "files": 0, "insertions": 0, "deletions": 0, "totalLines": total});
                rows.push(row.as_object().cloned().unwrap_or_default());
            }
        } else if let (Some(row), Some((add, rest))) = (rows.last_mut(), line.split_once('\t'))
            && let Some((del, _path)) = rest.split_once('\t')
        {
            // Binary files show `-` for both counts and add nothing.
            let (a, d) = (add.parse::<i64>().unwrap_or(0), del.parse::<i64>().unwrap_or(0));
            total += a - d;
            for (k, v) in [("files", 1), ("insertions", a), ("deletions", d)] {
                row[k] = json!(row[k].as_i64().unwrap_or(0) + v);
            }
            row["totalLines"] = json!(total);
        }
    }
    Value::Array(rows.into_iter().map(Value::Object).collect())
}

/// The task a commit belongs to: `[B03] …` or `chore(board): claim B03`.
fn task_of(subject: &str) -> Value {
    if let Some((id, _)) = subject.strip_prefix('[').and_then(|s| s.split_once(']')) {
        return json!(id);
    }
    let word = subject.split_whitespace().last().unwrap_or_default();
    let is_id = word.len() >= 3
        && word.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && word[1..].chars().all(|c| c.is_ascii_alphanumeric());
    if subject.starts_with("chore(board)") && is_id { json!(word) } else { Value::Null }
}

/// Runs `gh` and parses its JSON array; empty when `gh` is missing or signed out.
fn gh(args: &[&str]) -> Vec<Value> {
    let out = Command::new("gh").args(args).output();
    match out {
        Ok(o) if o.status.success() => serde_json::from_slice::<Vec<Value>>(&o.stdout).unwrap_or_default(),
        _ => {
            eprintln!("warning: `gh {}` failed; writing no rows", args.first().unwrap_or(&""));
            Vec::new()
        }
    }
}

/// Runs `git` in `root` and returns its stdout (empty on failure).
fn git(root: &Path, args: &[&str]) -> String {
    Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// The current UTC time as `YYYY-MM-DDTHH:MM:SSZ`.
fn now_iso() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_board_table() {
        let doc = "## Index\n\n| # | ID | Hard deps | Updated (UTC) |\n|---|---|---|---|\n| 1 | `T00` | — | 2026-10-27 05:15 |\n\nafter";
        let rows = table(doc, "## Index");
        assert_eq!(rows[0]["id"], "T00");
        assert_eq!(rows[0]["#"], "1");
        assert_eq!(rows[0]["updatedUtc"], "2026-10-27 05:15");
        assert_eq!(board_time("2026-10-27 05:15"), json!("2026-10-27T05:15:00Z"));
    }

    #[test]
    fn finds_the_task_of_a_commit() {
        assert_eq!(task_of("[B04] Server API"), json!("B04"));
        assert_eq!(task_of("chore(board): claim B04"), json!("B04"));
        assert_eq!(task_of("chore: remove the README"), Value::Null);
    }
}
