//! Delta compression — line-based diff with hashing.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOp {
    pub line: usize,
    pub op: String,  // "ADD" or "REM"
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaPatch {
    pub base_checksum: u64,
    pub current_checksum: u64,
    pub ops: Vec<DiffOp>,
}

/// Compute a simple hash for text content.
pub fn text_hash(text: &str) -> u64 {
    let mut hash: u64 = 5381;
    for b in text.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    hash
}

/// Compute delta between two text strings (line-based).
pub fn compute_delta(base: &str, current: &str) -> DeltaPatch {
    let base_lines: Vec<&str> = base.lines().collect();
    let curr_lines: Vec<&str> = current.lines().collect();
    let mut ops = Vec::new();
    let max_len = base_lines.len().max(curr_lines.len());

    for i in 0..max_len {
        match (base_lines.get(i), curr_lines.get(i)) {
            (Some(old), Some(new)) if old != new => {
                ops.push(DiffOp { line: i, op: "REM".into(), text: old.to_string() });
                ops.push(DiffOp { line: i, op: "ADD".into(), text: new.to_string() });
            }
            (None, Some(new)) => {
                ops.push(DiffOp { line: i, op: "ADD".into(), text: new.to_string() });
            }
            (Some(old), None) => {
                ops.push(DiffOp { line: i, op: "REM".into(), text: old.to_string() });
            }
            _ => {}
        }
    }

    DeltaPatch {
        base_checksum: text_hash(base),
        current_checksum: text_hash(current),
        ops,
    }
}

/// Apply a delta patch to base text.
pub fn apply_delta(base: &str, patch: &DeltaPatch) -> String {
    let mut lines: Vec<String> = base.lines().map(|s| s.to_string()).collect();
    for op in &patch.ops {
        match op.op.as_str() {
            "ADD" => {
                if op.line >= lines.len() {
                    lines.push(op.text.clone());
                } else {
                    lines[op.line] = op.text.clone();
                }
            }
            "REM" => {
                if op.line < lines.len() {
                    lines.remove(op.line);
                }
            }
            _ => {}
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_hash_deterministic() { let h1 = text_hash("hello"); let h2 = text_hash("hello"); assert_eq!(h1, h2); }
    #[test] fn test_hash_different() { let h1 = text_hash("hello"); let h2 = text_hash("world"); assert_ne!(h1, h2); }
    #[test] fn test_delta_no_change() { let d = compute_delta("abc", "abc"); assert!(d.ops.is_empty()); }
    #[test] fn test_delta_line_change() { let d = compute_delta("old line", "new line"); assert_eq!(d.ops.len(), 2); assert_eq!(d.ops[0].op, "REM"); assert_eq!(d.ops[1].op, "ADD"); }
    #[test] fn test_delta_addition() { let d = compute_delta("line1", "line1\nline2"); assert_eq!(d.ops.len(), 1); assert_eq!(d.ops[0].op, "ADD"); }
    #[test] fn test_delta_deletion() { let d = compute_delta("line1\nline2", "line1"); assert_eq!(d.ops.len(), 1); assert_eq!(d.ops[0].op, "REM"); }
    #[test] fn test_delta_checksums() { let d = compute_delta("a", "b"); assert_ne!(d.base_checksum, d.current_checksum); }
    #[test] fn test_apply_delta_change() { let base = "old"; let patch = compute_delta(base, "new"); let result = apply_delta(base, &patch); assert_eq!(result, "new"); }
    #[test] fn test_apply_delta_add() { let base = "line1"; let patch = compute_delta(base, "line1\nline2"); let result = apply_delta(base, &patch); assert!(result.contains("line2")); }
}
