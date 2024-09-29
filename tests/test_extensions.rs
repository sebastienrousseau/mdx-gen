#[cfg(test)]
mod tests {
    use mdx_gen::extensions::{process_custom_blocks, process_tables};
    use mdx_gen::{ColumnAlignment, CustomBlockType};

    #[test]
    fn test_column_alignment() {
        assert_eq!(ColumnAlignment::Left, ColumnAlignment::Left);
        assert_eq!(ColumnAlignment::Center, ColumnAlignment::Center);
        assert_eq!(ColumnAlignment::Right, ColumnAlignment::Right);
    }

    #[test]
    fn test_custom_block_get_alert_class() {
        assert_eq!(
            CustomBlockType::Note.get_alert_class(),
            "alert-info"
        );
        assert_eq!(
            CustomBlockType::Warning.get_alert_class(),
            "alert-warning"
        );
        assert_eq!(
            CustomBlockType::Tip.get_alert_class(),
            "alert-success"
        );
        assert_eq!(
            CustomBlockType::Info.get_alert_class(),
            "alert-primary"
        );
        assert_eq!(
            CustomBlockType::Important.get_alert_class(),
            "alert-danger"
        );
        assert_eq!(
            CustomBlockType::Caution.get_alert_class(),
            "alert-secondary"
        );
    }

    #[test]
    fn test_custom_block_get_title() {
        assert_eq!(CustomBlockType::Note.get_title(), "Note");
        assert_eq!(CustomBlockType::Warning.get_title(), "Warning");
        assert_eq!(CustomBlockType::Tip.get_title(), "Tip");
        assert_eq!(CustomBlockType::Info.get_title(), "Info");
        assert_eq!(CustomBlockType::Important.get_title(), "Important");
        assert_eq!(CustomBlockType::Caution.get_title(), "Caution");
    }

    #[test]
    fn test_process_custom_blocks() {
        let input = r#"
            <div class="note">This is a note.</div>
            <div class="WARNING">This is a warning.</div>
            <div class="Tip">This is a tip.</div>
            <div class="INFO">This is an info block.</div>
            <div class="Important">This is important.</div>
            <div class="caution">This is a caution.</div>
        "#;

        let processed = process_custom_blocks(input);

        assert!(processed.contains(r#"<div class="alert alert-info" role="alert"><strong>Note:</strong> This is a note.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-warning" role="alert"><strong>Warning:</strong> This is a warning.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-success" role="alert"><strong>Tip:</strong> This is a tip.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-primary" role="alert"><strong>Info:</strong> This is an info block.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-danger" role="alert"><strong>Important:</strong> This is important.</div>"#));
        assert!(processed.contains(r#"<div class="alert alert-secondary" role="alert"><strong>Caution:</strong> This is a caution.</div>"#));
    }

    #[test]
    fn test_process_tables() {
        let input = r#"<table><tr><td align="center">Center</td><td align="right">Right</td><td>Left</td></tr></table>"#;

        let processed = process_tables(input);

        assert!(processed.contains(
            r#"<div class="table-responsive"><table class="table">"#
        ));
        assert!(processed.contains(
            r#"<td align="center" class="text-center">Center</td>"#
        ));
        assert!(processed.contains(
            r#"<td align="right" class="text-right">Right</td>"#
        ));
        assert!(
            processed.contains(r#"<td class="text-left">Left</td>"#)
        );
        assert!(processed.contains("</table></div>"));
    }
}
