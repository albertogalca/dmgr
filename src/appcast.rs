use crate::changelog::ChangelogEntry;
use crate::runner;
use chrono::{DateTime, Utc};
use quick_xml::events::{BytesCData, BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::fs;
use std::io::Cursor;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AppcastItem {
    pub title: String,
    pub version: String,
    pub build: String,
    pub pub_date: String,
    pub download_url: String,
    pub length: u64,
    pub ed_signature: Option<String>,
    pub release_notes_html: String,
    pub minimum_system_version: Option<String>,
}

pub fn parse_appcast(path: &Path) -> Result<Vec<AppcastItem>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read appcast: {}", e))?;
    parse_appcast_content(&content)
}

pub fn parse_appcast_content(content: &str) -> Result<Vec<AppcastItem>, String> {
    use quick_xml::Reader;

    let mut reader = Reader::from_str(content);
    let mut items = Vec::new();
    let mut buf = Vec::new();

    let mut in_item = false;
    let mut in_description = false;
    let mut current_item: Option<AppcastItemBuilder> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "item" => {
                        in_item = true;
                        current_item = Some(AppcastItemBuilder::default());
                    }
                    "enclosure" if in_item => {
                        if let Some(ref mut item) = current_item {
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value =
                                    String::from_utf8_lossy(&attr.value).to_string();
                                match key.as_str() {
                                    "url" => item.download_url = Some(value),
                                    "length" => item.length = value.parse().ok(),
                                    "sparkle:version" => item.build = Some(value),
                                    "sparkle:shortVersionString" => item.version = Some(value),
                                    "sparkle:edSignature" => item.ed_signature = Some(value),
                                    _ => {}
                                }
                            }
                        }
                    }
                    "description" if in_item => {
                        in_description = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "item" => {
                        in_item = false;
                        if let Some(builder) = current_item.take() {
                            if let Some(item) = builder.build() {
                                items.push(item);
                            }
                        }
                    }
                    "description" => {
                        in_description = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_item && in_description {
                    if let Some(ref mut item) = current_item {
                        let text = e.unescape().unwrap_or_default().to_string();
                        item.release_notes_html = Some(text);
                    }
                }
            }
            Ok(Event::CData(ref e)) => {
                if in_item && in_description {
                    if let Some(ref mut item) = current_item {
                        let text = String::from_utf8_lossy(e.as_ref()).to_string();
                        item.release_notes_html = Some(text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(items)
}

#[derive(Default)]
struct AppcastItemBuilder {
    title: Option<String>,
    version: Option<String>,
    build: Option<String>,
    pub_date: Option<String>,
    download_url: Option<String>,
    length: Option<u64>,
    ed_signature: Option<String>,
    release_notes_html: Option<String>,
    minimum_system_version: Option<String>,
}

impl AppcastItemBuilder {
    fn build(self) -> Option<AppcastItem> {
        Some(AppcastItem {
            title: self.title.unwrap_or_default(),
            version: self.version?,
            build: self.build?,
            pub_date: self.pub_date.unwrap_or_default(),
            download_url: self.download_url?,
            length: self.length.unwrap_or(0),
            ed_signature: self.ed_signature,
            release_notes_html: self.release_notes_html.unwrap_or_default(),
            minimum_system_version: self.minimum_system_version,
        })
    }
}

pub fn generate_appcast(
    title: &str,
    description: &str,
    link: &str,
    items: &[AppcastItem],
) -> Result<String, String> {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|e| format!("XML write error: {}", e))?;

    let mut rss = BytesStart::new("rss");
    rss.push_attribute(("version", "2.0"));
    rss.push_attribute(("xmlns:sparkle", "http://www.andymatuschak.org/xml-namespaces/sparkle"));
    rss.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    writer
        .write_event(Event::Start(rss))
        .map_err(|e| format!("XML write error: {}", e))?;

    writer
        .write_event(Event::Start(BytesStart::new("channel")))
        .map_err(|e| format!("XML write error: {}", e))?;

    write_element(&mut writer, "title", title)?;
    write_element(&mut writer, "description", description)?;
    write_element(&mut writer, "link", link)?;
    write_element(&mut writer, "language", "en")?;

    for item in items {
        write_item(&mut writer, item)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("channel")))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::End(BytesEnd::new("rss")))
        .map_err(|e| format!("XML write error: {}", e))?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| format!("UTF-8 error: {}", e))
}

fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &str,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new(name)))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| format!("XML write error: {}", e))?;
    Ok(())
}

fn write_item<W: std::io::Write>(writer: &mut Writer<W>, item: &AppcastItem) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("item")))
        .map_err(|e| format!("XML write error: {}", e))?;

    write_element(writer, "title", &item.title)?;
    write_element(writer, "pubDate", &item.pub_date)?;

    writer
        .write_event(Event::Start(BytesStart::new("description")))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::CData(BytesCData::new(&item.release_notes_html)))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::End(BytesEnd::new("description")))
        .map_err(|e| format!("XML write error: {}", e))?;

    let mut enclosure = BytesStart::new("enclosure");
    enclosure.push_attribute(("url", item.download_url.as_str()));
    enclosure.push_attribute(("length", item.length.to_string().as_str()));
    enclosure.push_attribute(("type", "application/octet-stream"));
    enclosure.push_attribute(("sparkle:version", item.build.as_str()));
    enclosure.push_attribute(("sparkle:shortVersionString", item.version.as_str()));
    if let Some(ref sig) = item.ed_signature {
        enclosure.push_attribute(("sparkle:edSignature", sig.as_str()));
    }
    writer
        .write_event(Event::Empty(enclosure))
        .map_err(|e| format!("XML write error: {}", e))?;

    if let Some(ref min_ver) = item.minimum_system_version {
        write_element(writer, "sparkle:minimumSystemVersion", min_ver)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("item")))
        .map_err(|e| format!("XML write error: {}", e))?;

    Ok(())
}

pub fn merge_item(existing: &mut Vec<AppcastItem>, new_item: AppcastItem) -> Result<(), String> {
    // Check for duplicate
    if existing.iter().any(|i| i.build == new_item.build) {
        return Err(format!(
            "Duplicate build number in appcast: {}",
            new_item.build
        ));
    }

    let new_build: u64 = new_item.build.parse().unwrap_or(0);
    let pos = existing
        .iter()
        .position(|i| {
            let build: u64 = i.build.parse().unwrap_or(0);
            build < new_build
        })
        .unwrap_or(existing.len());

    existing.insert(pos, new_item);
    Ok(())
}

pub fn create_item(
    app_name: &str,
    version: &str,
    build: &str,
    download_url: &str,
    dmg_size: u64,
    ed_signature: Option<String>,
    changelog_entry: Option<&ChangelogEntry>,
    minimum_system_version: Option<String>,
) -> AppcastItem {
    let release_notes = changelog_entry
        .map(|e| e.content_html.clone())
        .unwrap_or_default();

    let pub_date = format_rfc2822(Utc::now());

    AppcastItem {
        title: format!("{} {}", app_name, version),
        version: version.to_string(),
        build: build.to_string(),
        pub_date,
        download_url: download_url.to_string(),
        length: dmg_size,
        ed_signature,
        release_notes_html: release_notes,
        minimum_system_version,
    }
}

fn format_rfc2822(dt: DateTime<Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S %z").to_string()
}

pub fn sign_dmg(dmg_path: &Path, private_key_path: Option<&Path>) -> Result<String, String> {
    let dmg_str = dmg_path.to_string_lossy();

    let mut args = vec![dmg_str.as_ref()];

    if let Some(key_path) = private_key_path {
        args.push("-f");
        args.push(key_path.to_str().ok_or("Invalid key path")?);
    }

    let output = runner::run_capture("sign_update", &args)
        .map_err(|e| format!("Failed to sign DMG: {}", e))?;

    Ok(output.trim().to_string())
}

pub fn write_appcast(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content).map_err(|e| format!("Failed to write appcast: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_item_ordering() {
        let mut items = vec![
            AppcastItem {
                title: "App 1.0".to_string(),
                version: "1.0".to_string(),
                build: "100".to_string(),
                pub_date: "".to_string(),
                download_url: "http://example.com/1.0.dmg".to_string(),
                length: 1000,
                ed_signature: None,
                release_notes_html: "".to_string(),
                minimum_system_version: None,
            },
        ];

        let new_item = AppcastItem {
            title: "App 1.1".to_string(),
            version: "1.1".to_string(),
            build: "110".to_string(),
            pub_date: "".to_string(),
            download_url: "http://example.com/1.1.dmg".to_string(),
            length: 1100,
            ed_signature: None,
            release_notes_html: "".to_string(),
            minimum_system_version: None,
        };

        merge_item(&mut items, new_item).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].build, "110"); // Newest first
        assert_eq!(items[1].build, "100");
    }

    #[test]
    fn test_merge_item_rejects_duplicate() {
        let mut items = vec![
            AppcastItem {
                title: "App 1.0".to_string(),
                version: "1.0".to_string(),
                build: "100".to_string(),
                pub_date: "".to_string(),
                download_url: "http://example.com/1.0.dmg".to_string(),
                length: 1000,
                ed_signature: None,
                release_notes_html: "".to_string(),
                minimum_system_version: None,
            },
        ];

        let duplicate = AppcastItem {
            title: "App 1.0 Updated".to_string(),
            version: "1.0".to_string(),
            build: "100".to_string(), // Same build
            pub_date: "".to_string(),
            download_url: "http://example.com/1.0-new.dmg".to_string(),
            length: 1000,
            ed_signature: None,
            release_notes_html: "".to_string(),
            minimum_system_version: None,
        };

        let result = merge_item(&mut items, duplicate);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate"));
    }
}
