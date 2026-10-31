use changeradar_hashing::{hash_bytes, perceptual_hash_image, perceptual_similarity};
use changeradar_types::{
    BinaryDiffData, ComparatorType, ComparisonOutput, CsvDiffData, CsvRowChange, ImageDiffData,
    JsonDiffItem, SideBySideLine, TextDiffData,
};
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComparisonError {
    #[error("Comparison failed: {0}")]
    Failed(String),
}

pub struct UniversalComparator;

impl UniversalComparator {
    /// Automatically select the best comparison engine based on file extension and content.
    pub fn compare(file_name: &str, bytes_a: &[u8], bytes_b: &[u8]) -> ComparisonOutput {
        let ext = file_name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "json" => {
                if let Ok(out) = Self::compare_json(bytes_a, bytes_b) {
                    return out;
                }
                Self::compare_text(bytes_a, bytes_b)
            }
            "csv" | "tsv" => {
                let delimiter = if ext == "tsv" { b'\t' } else { b',' };
                if let Ok(out) = Self::compare_csv(bytes_a, bytes_b, delimiter) {
                    return out;
                }
                Self::compare_text(bytes_a, bytes_b)
            }
            "png" | "jpg" | "jpeg" | "webp" | "bmp" => {
                if let Ok(out) = Self::compare_image(bytes_a, bytes_b) {
                    return out;
                }
                Self::compare_binary(bytes_a, bytes_b)
            }
            "pdf" => Self::compare_pdf(bytes_a, bytes_b),
            _ => {
                // If content looks like valid UTF-8, treat as text
                if let (Ok(_), Ok(_)) = (std::str::from_utf8(bytes_a), std::str::from_utf8(bytes_b)) {
                    Self::compare_text(bytes_a, bytes_b)
                } else {
                    Self::compare_binary(bytes_a, bytes_b)
                }
            }
        }
    }

    /// Compare plain text, source code, markdown, config files using Myers diff.
    pub fn compare_text(bytes_a: &[u8], bytes_b: &[u8]) -> ComparisonOutput {
        let text_a = String::from_utf8_lossy(bytes_a);
        let text_b = String::from_utf8_lossy(bytes_b);

        let diff = TextDiff::from_lines(&text_a, &text_b);

        let mut added = 0;
        let mut removed = 0;
        let mut unified = String::new();
        let mut side_by_side = Vec::new();

        let mut line_a = 1;
        let mut line_b = 1;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Delete => {
                    removed += 1;
                    unified.push_str(&format!("-{}", change.value()));
                    side_by_side.push(SideBySideLine {
                        line_num_a: Some(line_a),
                        line_text_a: Some(change.value().trim_end().to_string()),
                        line_num_b: None,
                        line_text_b: None,
                        change_type: "DELETE".to_string(),
                    });
                    line_a += 1;
                }
                ChangeTag::Insert => {
                    added += 1;
                    unified.push_str(&format!("+{}", change.value()));
                    side_by_side.push(SideBySideLine {
                        line_num_a: None,
                        line_text_a: None,
                        line_num_b: Some(line_b),
                        line_text_b: Some(change.value().trim_end().to_string()),
                        change_type: "INSERT".to_string(),
                    });
                    line_b += 1;
                }
                ChangeTag::Equal => {
                    unified.push_str(&format!(" {}", change.value()));
                    side_by_side.push(SideBySideLine {
                        line_num_a: Some(line_a),
                        line_text_a: Some(change.value().trim_end().to_string()),
                        line_num_b: Some(line_b),
                        line_text_b: Some(change.value().trim_end().to_string()),
                        change_type: "EQUAL".to_string(),
                    });
                    line_a += 1;
                    line_b += 1;
                }
            }
        }

        let summary = format!("{} lines added, {} lines removed", added, removed);

        ComparisonOutput {
            comparator_type: ComparatorType::Text,
            summary,
            added_count: added,
            removed_count: removed,
            modified_count: 0,
            text_diff: Some(TextDiffData {
                unified_diff: unified,
                side_by_side,
            }),
            json_diff: None,
            csv_diff: None,
            image_diff: None,
            binary_diff: None,
        }
    }

    /// Structural comparison of JSON values.
    pub fn compare_json(bytes_a: &[u8], bytes_b: &[u8]) -> Result<ComparisonOutput, ComparisonError> {
        let val_a: serde_json::Value = serde_json::from_slice(bytes_a)
            .map_err(|e| ComparisonError::Failed(format!("Invalid JSON in original: {}", e)))?;
        let val_b: serde_json::Value = serde_json::from_slice(bytes_b)
            .map_err(|e| ComparisonError::Failed(format!("Invalid JSON in modified: {}", e)))?;

        let mut diff_items = Vec::new();
        Self::diff_json_values("$", &val_a, &val_b, &mut diff_items);

        let mut added = 0;
        let mut removed = 0;
        let mut modified = 0;

        for item in &diff_items {
            match item.kind.as_str() {
                "ADDED" => added += 1,
                "REMOVED" => removed += 1,
                _ => modified += 1,
            }
        }

        let summary = format!(
            "JSON differences: {} added, {} removed, {} modified fields",
            added, removed, modified
        );

        Ok(ComparisonOutput {
            comparator_type: ComparatorType::Json,
            summary,
            added_count: added,
            removed_count: removed,
            modified_count: modified,
            text_diff: None,
            json_diff: Some(diff_items),
            csv_diff: None,
            image_diff: None,
            binary_diff: None,
        })
    }

    fn diff_json_values(
        path: &str,
        a: &serde_json::Value,
        b: &serde_json::Value,
        diffs: &mut Vec<JsonDiffItem>,
    ) {
        use serde_json::Value;

        if a == b {
            return;
        }

        match (a, b) {
            (Value::Object(map_a), Value::Object(map_b)) => {
                let keys_a: HashSet<&String> = map_a.keys().collect();
                let keys_b: HashSet<&String> = map_b.keys().collect();

                // Removed keys
                for k in keys_a.difference(&keys_b) {
                    diffs.push(JsonDiffItem {
                        path: format!("{}.{}", path, k),
                        kind: "REMOVED".to_string(),
                        old_val: Some(map_a[*k].to_string()),
                        new_val: None,
                    });
                }

                // Added keys
                for k in keys_b.difference(&keys_a) {
                    diffs.push(JsonDiffItem {
                        path: format!("{}.{}", path, k),
                        kind: "ADDED".to_string(),
                        old_val: None,
                        new_val: Some(map_b[*k].to_string()),
                    });
                }

                // Common keys
                for k in keys_a.intersection(&keys_b) {
                    let sub_path = format!("{}.{}", path, k);
                    Self::diff_json_values(&sub_path, &map_a[*k], &map_b[*k], diffs);
                }
            }
            (Value::Array(arr_a), Value::Array(arr_b)) => {
                let max_len = arr_a.len().max(arr_b.len());
                for i in 0..max_len {
                    let sub_path = format!("{}[{}]", path, i);
                    match (arr_a.get(i), arr_b.get(i)) {
                        (Some(item_a), Some(item_b)) => {
                            Self::diff_json_values(&sub_path, item_a, item_b, diffs);
                        }
                        (Some(item_a), None) => {
                            diffs.push(JsonDiffItem {
                                path: sub_path,
                                kind: "REMOVED".to_string(),
                                old_val: Some(item_a.to_string()),
                                new_val: None,
                            });
                        }
                        (None, Some(item_b)) => {
                            diffs.push(JsonDiffItem {
                                path: sub_path,
                                kind: "ADDED".to_string(),
                                old_val: None,
                                new_val: Some(item_b.to_string()),
                            });
                        }
                        (None, None) => {}
                    }
                }
            }
            _ => {
                diffs.push(JsonDiffItem {
                    path: path.to_string(),
                    kind: "MODIFIED".to_string(),
                    old_val: Some(a.to_string()),
                    new_val: Some(b.to_string()),
                });
            }
        }
    }

    /// Compare CSV tables by rows and headers.
    pub fn compare_csv(
        bytes_a: &[u8],
        bytes_b: &[u8],
        delimiter: u8,
    ) -> Result<ComparisonOutput, ComparisonError> {
        let mut rdr_a = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(true)
            .from_reader(bytes_a);

        let mut rdr_b = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(true)
            .from_reader(bytes_b);

        let headers_a: Vec<String> = rdr_a
            .headers()
            .map_err(|e| ComparisonError::Failed(e.to_string()))?
            .iter()
            .map(|s| s.to_string())
            .collect();

        let rows_a: Vec<Vec<String>> = rdr_a
            .records()
            .filter_map(|r| r.ok())
            .map(|r| r.iter().map(|s| s.to_string()).collect())
            .collect();

        let rows_b: Vec<Vec<String>> = rdr_b
            .records()
            .filter_map(|r| r.ok())
            .map(|r| r.iter().map(|s| s.to_string()).collect())
            .collect();

        let mut added_rows = 0;
        let mut removed_rows = 0;
        let mut modified_rows = 0;
        let mut sample_changes = Vec::new();

        let max_rows = rows_a.len().max(rows_b.len());

        for i in 0..max_rows {
            match (rows_a.get(i), rows_b.get(i)) {
                (Some(ra), Some(rb)) => {
                    if ra != rb {
                        modified_rows += 1;
                        if sample_changes.len() < 100 {
                            sample_changes.push(CsvRowChange {
                                row_index: i + 1,
                                change_kind: "MODIFIED".to_string(),
                                row_data: rb.clone(),
                            });
                        }
                    }
                }
                (Some(ra), None) => {
                    removed_rows += 1;
                    if sample_changes.len() < 100 {
                        sample_changes.push(CsvRowChange {
                            row_index: i + 1,
                            change_kind: "REMOVED".to_string(),
                            row_data: ra.clone(),
                        });
                    }
                }
                (None, Some(rb)) => {
                    added_rows += 1;
                    if sample_changes.len() < 100 {
                        sample_changes.push(CsvRowChange {
                            row_index: i + 1,
                            change_kind: "ADDED".to_string(),
                            row_data: rb.clone(),
                        });
                    }
                }
                (None, None) => {}
            }
        }

        let summary = format!(
            "CSV differences: {} rows added, {} rows removed, {} rows modified",
            added_rows, removed_rows, modified_rows
        );

        Ok(ComparisonOutput {
            comparator_type: ComparatorType::Csv,
            summary,
            added_count: added_rows,
            removed_count: removed_rows,
            modified_count: modified_rows,
            text_diff: None,
            json_diff: None,
            csv_diff: Some(CsvDiffData {
                headers: headers_a,
                rows_added: added_rows,
                rows_removed: removed_rows,
                rows_modified: modified_rows,
                sample_changes,
            }),
            image_diff: None,
            binary_diff: None,
        })
    }

    /// Compare images visually and perceptually.
    pub fn compare_image(bytes_a: &[u8], bytes_b: &[u8]) -> Result<ComparisonOutput, ComparisonError> {
        let img_a = image::load_from_memory(bytes_a)
            .map_err(|e| ComparisonError::Failed(format!("Failed to decode image A: {}", e)))?;
        let img_b = image::load_from_memory(bytes_b)
            .map_err(|e| ComparisonError::Failed(format!("Failed to decode image B: {}", e)))?;

        let (w_a, h_a) = (img_a.width(), img_a.height());
        let (w_b, h_b) = (img_b.width(), img_b.height());
        let dimension_match = w_a == w_b && h_a == h_b;

        let exact_match = bytes_a == bytes_b;

        let phash_a = perceptual_hash_image(bytes_a).unwrap_or(0);
        let phash_b = perceptual_hash_image(bytes_b).unwrap_or(0);
        let similarity = perceptual_similarity(phash_a, phash_b);

        let summary = if exact_match {
            "Images are identical".to_string()
        } else if !dimension_match {
            format!(
                "Image dimensions changed from {}x{} to {}x{} (perceptual similarity: {:.1}%)",
                w_a, h_a, w_b, h_b, similarity
            )
        } else {
            format!(
                "Image content changed at {}x{} (perceptual similarity: {:.1}%)",
                w_a, h_a, similarity
            )
        };

        Ok(ComparisonOutput {
            comparator_type: ComparatorType::Image,
            summary,
            added_count: if exact_match { 0 } else { 1 },
            removed_count: 0,
            modified_count: if exact_match { 0 } else { 1 },
            text_diff: None,
            json_diff: None,
            csv_diff: None,
            image_diff: Some(ImageDiffData {
                old_width: w_a,
                old_height: h_a,
                new_width: w_b,
                new_height: h_b,
                dimension_match,
                perceptual_similarity_pct: similarity,
                exact_match,
            }),
            binary_diff: None,
        })
    }

    /// Extract text streams from PDF and perform line diff.
    pub fn compare_pdf(bytes_a: &[u8], bytes_b: &[u8]) -> ComparisonOutput {
        let text_a = Self::extract_pdf_text_heuristic(bytes_a);
        let text_b = Self::extract_pdf_text_heuristic(bytes_b);

        let text_out = Self::compare_text(text_a.as_bytes(), text_b.as_bytes());

        ComparisonOutput {
            comparator_type: ComparatorType::Pdf,
            summary: format!("PDF text comparison: {}", text_out.summary),
            added_count: text_out.added_count,
            removed_count: text_out.removed_count,
            modified_count: text_out.modified_count,
            text_diff: text_out.text_diff,
            json_diff: None,
            csv_diff: None,
            image_diff: None,
            binary_diff: None,
        }
    }

    fn extract_pdf_text_heuristic(bytes: &[u8]) -> String {
        // Extract readable ASCII/UTF-8 string literals between BT and ET operators or parentheses
        let mut extracted = String::new();
        let mut in_paren = false;
        let mut buf = Vec::new();

        for &b in bytes {
            if b == b'(' {
                in_paren = true;
                buf.clear();
            } else if b == b')' && in_paren {
                in_paren = false;
                if buf.iter().all(|&c| (32..=126).contains(&c) || c == b'\n' || c == b'\r') {
                    if let Ok(s) = std::str::from_utf8(&buf) {
                        extracted.push_str(s);
                        extracted.push('\n');
                    }
                }
            } else if in_paren {
                buf.push(b);
            }
        }

        if extracted.trim().is_empty() {
            format!("(Extracted {} raw PDF bytes)", bytes.len())
        } else {
            extracted
        }
    }

    /// Binary fallback diff.
    pub fn compare_binary(bytes_a: &[u8], bytes_b: &[u8]) -> ComparisonOutput {
        let hash_a = hash_bytes(bytes_a);
        let hash_b = hash_bytes(bytes_b);
        let len_a = bytes_a.len() as u64;
        let len_b = bytes_b.len() as u64;
        let diff_bytes = (len_b as i64) - (len_a as i64);

        let mut hex_samples = Vec::new();
        let sample_limit = 16.min(bytes_a.len()).min(bytes_b.len());

        if sample_limit > 0 {
            hex_samples.push(format!(
                "First bytes A: {:02x?}",
                &bytes_a[0..sample_limit]
            ));
            hex_samples.push(format!(
                "First bytes B: {:02x?}",
                &bytes_b[0..sample_limit]
            ));
        }

        let summary = if hash_a == hash_b {
            "Binary files have identical content".to_string()
        } else {
            format!(
                "Binary files differ: size changed by {} bytes",
                diff_bytes
            )
        };

        ComparisonOutput {
            comparator_type: ComparatorType::Binary,
            summary,
            added_count: if hash_a == hash_b { 0 } else { 1 },
            removed_count: 0,
            modified_count: if hash_a == hash_b { 0 } else { 1 },
            text_diff: None,
            json_diff: None,
            csv_diff: None,
            image_diff: None,
            binary_diff: Some(BinaryDiffData {
                old_size: len_a,
                new_size: len_b,
                size_diff: diff_bytes,
                old_hash: hash_a,
                new_hash: hash_b,
                hex_sample_diff: hex_samples,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_comparison() {
        let a = b"line 1\nline 2\nline 3\n";
        let b = b"line 1\nline 2 modified\nline 3\nline 4\n";
        let res = UniversalComparator::compare_text(a, b);
        assert_eq!(res.added_count, 2);
        assert_eq!(res.removed_count, 1);
        assert!(res.text_diff.is_some());
    }

    #[test]
    fn test_json_comparison() {
        let a = br#"{"name":"Ajay","age":25,"roles":["user"]}"#;
        let b = br#"{"name":"Alex","age":25,"roles":["user","admin"]}"#;
        let res = UniversalComparator::compare_json(a, b).unwrap();
        assert_eq!(res.modified_count, 1); // name
        assert_eq!(res.added_count, 1); // roles[1]
    }

    #[test]
    fn test_csv_comparison() {
        let a = b"id,name,role\n1,Alice,Dev\n2,Bob,QA\n";
        let b = b"id,name,role\n1,Alice,Lead\n2,Bob,QA\n3,Charlie,PM\n";
        let res = UniversalComparator::compare_csv(a, b, b',').unwrap();
        assert_eq!(res.added_count, 1);
        assert_eq!(res.modified_count, 1);
    }
}
