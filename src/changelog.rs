use pulldown_cmark::{html, Parser};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    pub version: String,
    #[allow(dead_code)]
    pub build: Option<String>,
    #[allow(dead_code)]
    pub date: Option<String>,
    pub content_markdown: String,
    pub content_html: String,
}

pub fn parse_changelog(path: &Path) -> Result<Vec<ChangelogEntry>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read changelog: {}", e))?;

    parse_changelog_content(&content)
}

pub fn parse_changelog_content(content: &str) -> Result<Vec<ChangelogEntry>, String> {
    let mut entries = Vec::new();
    let mut current_entry: Option<(String, Option<String>, Option<String>, String)> = None;

    for line in content.lines() {
        if line.starts_with("## ") {
            if let Some((version, build, date, md_content)) = current_entry.take() {
                let html_content = markdown_to_html(&md_content);
                entries.push(ChangelogEntry {
                    version,
                    build,
                    date,
                    content_markdown: md_content.trim().to_string(),
                    content_html: html_content,
                });
            }

            let header = &line[3..];
            if let Some(parsed) = parse_version_header(header) {
                current_entry = Some((parsed.0, parsed.1, parsed.2, String::new()));
            }
        } else if current_entry.is_some() {
            if let Some((_, _, _, ref mut content)) = current_entry {
                content.push_str(line);
                content.push('\n');
            }
        }
    }

    if let Some((version, build, date, md_content)) = current_entry {
        let html_content = markdown_to_html(&md_content);
        entries.push(ChangelogEntry {
            version,
            build,
            date,
            content_markdown: md_content.trim().to_string(),
            content_html: html_content,
        });
    }

    Ok(entries)
}

fn parse_version_header(header: &str) -> Option<(String, Option<String>, Option<String>)> {
    let header = header.trim();

    if let Some(paren_start) = header.find('(') {
        if let Some(paren_end) = header.find(')') {
            let version = header[..paren_start].trim().to_string();
            let build = header[paren_start + 1..paren_end].trim().to_string();
            return Some((version, Some(build), None));
        }
    }

    if header.starts_with('[') {
        if let Some(bracket_end) = header.find(']') {
            let version = header[1..bracket_end].trim().to_string();
            let rest = header[bracket_end + 1..].trim();
            let date = if rest.starts_with('-') || rest.starts_with('–') {
                Some(rest[1..].trim().to_string())
            } else {
                None
            };
            return Some((version, None, date));
        }
    }

    if let Some(dash_pos) = header.find(" - ") {
        let version = header[..dash_pos].trim().to_string();
        let date = header[dash_pos + 3..].trim().to_string();
        return Some((version, None, Some(date)));
    }

    if !header.is_empty() {
        return Some((header.to_string(), None, None));
    }

    None
}

pub fn find_entry_for_version<'a>(
    entries: &'a [ChangelogEntry],
    version: &str,
) -> Option<&'a ChangelogEntry> {
    entries.iter().find(|e| e.version == version)
}

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_build_format() {
        let result = parse_version_header("1.2.3 (456)");
        assert!(result.is_some());
        let (version, build, date) = result.unwrap();
        assert_eq!(version, "1.2.3");
        assert_eq!(build, Some("456".to_string()));
        assert_eq!(date, None);
    }

    #[test]
    fn test_parse_bracketed_version_date() {
        let result = parse_version_header("[1.2.3] - 2024-01-15");
        assert!(result.is_some());
        let (version, build, date) = result.unwrap();
        assert_eq!(version, "1.2.3");
        assert_eq!(build, None);
        assert_eq!(date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_parse_simple_version_date() {
        let result = parse_version_header("1.2.3 - 2024-01-15");
        assert!(result.is_some());
        let (version, build, date) = result.unwrap();
        assert_eq!(version, "1.2.3");
        assert_eq!(build, None);
        assert_eq!(date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_parse_simple_version() {
        let result = parse_version_header("1.2.3");
        assert!(result.is_some());
        let (version, build, date) = result.unwrap();
        assert_eq!(version, "1.2.3");
        assert_eq!(build, None);
        assert_eq!(date, None);
    }

    #[test]
    fn test_parse_changelog_content() {
        let content = r#"# Changelog

## 1.2.0 (100)

- Added new feature
- Fixed bug

## 1.1.0 (90)

- Initial release
"#;
        let entries = parse_changelog_content(content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].version, "1.2.0");
        assert_eq!(entries[0].build, Some("100".to_string()));
        assert!(entries[0].content_html.contains("<li>"));
        assert_eq!(entries[1].version, "1.1.0");
    }
}
