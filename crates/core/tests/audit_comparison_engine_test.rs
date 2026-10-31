use changeradar_comparison::UniversalComparator;
use changeradar_types::ComparatorType;

#[test]
fn test_comparison_all_six_engines() {
    // 1. Text Comparison: insertions, deletions, empty, Unicode, line endings
    let empty_a = b"";
    let empty_b = b"";
    let diff_empty = UniversalComparator::compare("test.txt", empty_a, empty_b);
    assert_eq!(diff_empty.added_count, 0);
    assert_eq!(diff_empty.removed_count, 0);

    let unicode_a = "Привет, мир!\nLine 2\n".as_bytes();
    let unicode_b = "Привет, ChangeRadar!\nLine 2 modified\nLine 3 added\n".as_bytes();
    let diff_uni = UniversalComparator::compare("test.txt", unicode_a, unicode_b);
    assert!(diff_uni.added_count >= 1);
    assert!(diff_uni.text_diff.is_some());

    // 2. JSON Comparison: nested objects, arrays, value types, invalid JSON
    let json_a =
        br#"{"title":"Paper","authors":["Alice"],"meta":{"citations":10,"published":true}}"#;
    let json_b = br#"{"title":"Paper V2","authors":["Alice","Bob"],"meta":{"citations":15,"published":true}}"#;
    let json_diff = UniversalComparator::compare_json(json_a, json_b).unwrap();
    assert_eq!(json_diff.comparator_type, ComparatorType::Json);
    assert_eq!(json_diff.modified_count, 2); // title and meta.citations
    assert_eq!(json_diff.added_count, 1); // authors[1]

    let invalid_json = br#"{"title": unquoted"#;
    let err_json = UniversalComparator::compare_json(json_a, invalid_json);
    assert!(err_json.is_err(), "Invalid JSON must return error");

    // 3. CSV Comparison: rows, headers, custom delimiters (TSV)
    let csv_a = b"id,name,val\n1,alpha,100\n2,beta,200\n";
    let csv_b = b"id,name,val\n1,alpha,100\n2,beta,250\n3,gamma,300\n";
    let csv_diff = UniversalComparator::compare_csv(csv_a, csv_b, b',').unwrap();
    assert_eq!(csv_diff.modified_count, 1); // beta val changed
    assert_eq!(csv_diff.added_count, 1); // gamma added

    let tsv_a = b"id\tname\n1\tItemA\n";
    let tsv_b = b"id\tname\n1\tItemA\n2\tItemB\n";
    let tsv_diff = UniversalComparator::compare("data.tsv", tsv_a, tsv_b);
    assert_eq!(tsv_diff.comparator_type, ComparatorType::Csv);

    // 4. Image Comparison: identical, different dimensions, perceptual similarity
    // 1x1 PNG bytes
    let png_1x1 = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let img_identical = UniversalComparator::compare_image(&png_1x1, &png_1x1).unwrap();
    assert_eq!(img_identical.comparator_type, ComparatorType::Image);
    assert!(img_identical.image_diff.as_ref().unwrap().exact_match);

    // 5. PDF Comparison: text heuristic extraction
    let pdf_a = b"%PDF-1.4\n1 0 obj\n<< /Length 20 >>\nstream\n(Hello ChangeRadar PDF Document)\nendstream\nendobj\n";
    let pdf_b = b"%PDF-1.4\n1 0 obj\n<< /Length 28 >>\nstream\n(Hello ChangeRadar PDF Document V2)\nendstream\nendobj\n";
    let pdf_diff = UniversalComparator::compare("document.pdf", pdf_a, pdf_b);
    assert_eq!(pdf_diff.comparator_type, ComparatorType::Pdf);

    // 6. Binary Comparison: different bytes, hex samples
    let bin_a = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02];
    let bin_b = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xFF, 0xFE];
    let bin_diff = UniversalComparator::compare("firmware.bin", &bin_a, &bin_b);
    assert_eq!(bin_diff.comparator_type, ComparatorType::Binary);
    assert!(bin_diff.binary_diff.is_some());
    let b_info = bin_diff.binary_diff.unwrap();
    assert_eq!(b_info.old_size, 6);
    assert_eq!(b_info.new_size, 6);
    assert_ne!(b_info.old_hash, b_info.new_hash);
}
