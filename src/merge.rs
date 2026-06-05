//! Three-way merge — reconcile concurrent edits against a common base.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MergeLine {
    Clean(String),
    Conflict { ours: String, theirs: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub has_conflicts: bool,
    pub lines: Vec<MergeLine>,
    pub clean_count: usize,
    pub conflict_count: usize,
}

/// Three-way merge: reconcile ours and theirs against a common base.
pub fn three_way_merge(base: &str, ours: &str, theirs: &str) -> MergeResult {
    let base_lines: Vec<&str> = base.lines().collect();
    let ours_lines: Vec<&str> = ours.lines().collect();
    let theirs_lines: Vec<&str> = theirs.lines().collect();

    let mut lines = Vec::new();
    let mut has_conflicts = false;
    let mut clean_count = 0;
    let mut conflict_count = 0;

    let max_len = ours_lines.len().max(theirs_lines.len());
    for i in 0..max_len {
        let b = base_lines.get(i).copied();
        let o = ours_lines.get(i).copied();
        let t = theirs_lines.get(i).copied();

        if o == t {
            // Same change (or both unchanged) — take either
            if let Some(line) = o {
                lines.push(MergeLine::Clean(line.to_string()));
                clean_count += 1;
            }
        } else if o != b && t == b {
            // Only ours changed — take ours
            if let Some(line) = o {
                lines.push(MergeLine::Clean(line.to_string()));
                clean_count += 1;
            }
        } else if t != b && o == b {
            // Only theirs changed — take theirs
            if let Some(line) = t {
                lines.push(MergeLine::Clean(line.to_string()));
                clean_count += 1;
            }
        } else {
            // Both changed differently — conflict
            has_conflicts = true;
            lines.push(MergeLine::Conflict {
                ours: o.unwrap_or("").to_string(),
                theirs: t.unwrap_or("").to_string(),
            });
            conflict_count += 1;
        }
    }

    MergeResult { has_conflicts, lines, clean_count, conflict_count }
}

/// Render merge result to text with conflict markers.
pub fn render_merge(result: &MergeResult) -> String {
    let mut out = String::new();
    for line in &result.lines {
        match line {
            MergeLine::Clean(text) => out.push_str(&format!("{}\n", text)),
            MergeLine::Conflict { ours, theirs } => {
                out.push_str(&format!("<<<<<<< OURS\n{}\n=======\n{}\n>>>>>>> THEIRS\n", ours, theirs));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_merge_no_changes() { let r = three_way_merge("a\nb", "a\nb", "a\nb"); assert!(!r.has_conflicts); assert_eq!(r.clean_count, 2); }
    #[test] fn test_merge_ours_only() { let r = three_way_merge("a", "b", "a"); assert!(!r.has_conflicts); assert_eq!(r.lines[0], MergeLine::Clean("b".into())); }
    #[test] fn test_merge_theirs_only() { let r = three_way_merge("a", "a", "c"); assert!(!r.has_conflicts); assert_eq!(r.lines[0], MergeLine::Clean("c".into())); }
    #[test] fn test_merge_both_same() { let r = three_way_merge("a", "b", "b"); assert!(!r.has_conflicts); }
    #[test] fn test_merge_conflict() { let r = three_way_merge("a", "b", "c"); assert!(r.has_conflicts); assert_eq!(r.conflict_count, 1); }
    #[test] fn test_merge_mixed() { let r = three_way_merge("a\nb\nc", "a\nx\nc", "a\nb\ny"); assert!(!r.has_conflicts); assert_eq!(r.clean_count, 3); }
    #[test] fn test_render_clean() { let r = three_way_merge("a", "b", "b"); let text = render_merge(&r); assert!(!text.contains("<<<<<")); }
    #[test] fn test_render_conflict() { let r = three_way_merge("a", "b", "c"); let text = render_merge(&r); assert!(text.contains("OURS")); assert!(text.contains("THEIRS")); }
    #[test] fn test_merge_counts() { let r = three_way_merge("a\nb\nc", "d\nb\ne", "a\nf\ne"); assert_eq!(r.clean_count + r.conflict_count, 3); }
}
