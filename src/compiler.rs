use gray_matter::Matter;
use gray_matter::engine::YAML;
use walkdir::WalkDir;
use serde_json::{Map, Value};
use std::{path::Path, io::{Result, self}, fs::{self, create_dir_all}};
use comrak::{markdown_to_html, ComrakOptions};

const DEFAULT_HEAD: &str = "<head>\
        <meta charset=\"utf-8\">\
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
        <link rel=\"icon\" href=\"%root_dir%/favicon.png\">\
        <link rel=\"stylesheet\" href=\"%root_dir%/style.css\">\
        <title>%title%</title>\
        <meta name=\"description\" content=\"%description%\">\
    </head>";

fn generate_head_html(template: &str, args: Map<String, Value>) -> String {
    let mut head = String::from(template);

    for (key, value) in args.iter() {
        head = head.replace(&format!("%{}%", key), value.as_str().unwrap_or(""));
    }

    head
}

pub struct MarkdownFile {
    path: String,
    body: String
}

pub fn get_files(source_dir: &str) -> Vec<MarkdownFile> {
    let mut markdown_files: Vec<MarkdownFile> = Vec::new();
    
    let walker = WalkDir::new(source_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().expect("Couldn't fetch metadata").is_file())
        .filter(|e| e.file_name().to_str().expect("Couldn't convert filename to string").ends_with(".md"))
        .filter(|e| !e.file_name().to_str().unwrap().starts_with("_"));


    for entry in walker {
        let path = entry.path();
        let file_contents = fs::read_to_string(path);

        match file_contents {
            Ok(contents) => {
                markdown_files.push(MarkdownFile {
                    path: path.to_str().unwrap().to_string(),
                    body: contents
                });
            }
            Err(_) => ()
        }
    };

    return markdown_files;
}


pub fn generate_build_dir(build_dir: &str, source_dir: &str) -> Result<()> {
    println!("Creating build folder...");

    let _ = remove_dir_contents(&build_dir);

    fs::create_dir_all(&build_dir)
        .expect("Couldn't generate build directory!");

    let walker = WalkDir::new(source_dir).into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_name().to_str().unwrap().starts_with("_"))
        .filter(|e| !e.file_name().to_str().unwrap().ends_with(".md"))
        .filter(|e| e.metadata().expect("Couldn't fetch metadata").is_file());

    for entry in walker {
        let path = entry.path();
        let file_path = path.to_str().unwrap().to_string()
            .replace(source_dir, "");

        let file_path = if file_path.starts_with("/") {
            file_path[1..].to_string()
        } else {
            file_path
        };

        let new_path = Path::new(build_dir).join(file_path);

        println!("{}", new_path.to_str().unwrap());

        match new_path.parent() {
            Some(parent) => {
                fs::create_dir_all(parent)?;
            }
            None => ()
        }

        fs::copy(path, new_path)?;
    }

    Ok(())
}

pub fn generate_markdown_files(markdown_files: &Vec<MarkdownFile>, build_dir: &str, source_dir: &str, root_dir: &str, head: &str, nav: &str, footer: &str) {
    for md_file in markdown_files.iter() {
        let path = Path::new(&md_file.path);
        
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&md_file.body);

        let file_path = path.to_str().unwrap().to_string().replace(source_dir, "");

        let file_path = if file_path.starts_with("/") {
            file_path[1..].to_string()
        } else {
            file_path
        };

        let metadata: Map<String, Value> = match result.data {
            Some(parsed) => {
                match parsed.deserialize() {
                    Ok(fm) => fm,
                    Err(_) => Map::new()
                }
            }
            None => Map::new()
        };

        let mut html = String::from("<html>");
        let mut head_args = metadata.clone();
        head_args.insert("root_dir".to_string(), Value::String(root_dir.to_string()));

        let head_html = if head.is_empty() {
            generate_head_html(DEFAULT_HEAD, head_args)
        } else {
            generate_head_html(head, head_args)
        };

        html.push_str(&head_html);
        html.push_str("<body><main>");

        if nav != "" && metadata.get("nav") != Some(&serde_json::Value::Bool(false)) {
            html.push_str(nav);
        }

        html.push_str(&markdown_to_html(&result.content, &ComrakOptions {
            extension: comrak::ComrakExtensionOptions {
                strikethrough: true,
                table: true,
                autolink: true,
                tasklist: true,
                superscript: false,
                ..Default::default()
            },
            render: comrak::ComrakRenderOptions {
                unsafe_: true,
                ..Default::default()
            },
            ..Default::default()
        }));
        html.push_str("</main>");
        
        if footer != "" && metadata.get("footer") != Some(&serde_json::Value::Bool(false)) {
            html.push_str(footer);
        }

        html.push_str("</body></html>");

        let new_path = 
            build_dir.to_owned() + 
            "/" + 
            &file_path.replace(".md", ".html");

        println!("{}", new_path);

        match Path::new(&new_path).parent() {
            Some(parent) => {
                let _ = create_dir_all(parent);
            }
            None => ()
        }

        fs::write(new_path, html).unwrap();
    }
}

pub fn generate_footer(pages_dir: &str) -> String {
    if Path::new(pages_dir).join("_footer.html").exists() {
        let footer = fs::read_to_string(Path::new(pages_dir).join("_footer.html"))
            .expect("Couldn't read footer file");

        return footer;
    } else if Path::new(pages_dir).join("_footer.md").exists() {
        let footer = fs::read_to_string(Path::new(pages_dir).join("_footer.md"))
            .expect("Couldn't read footer file");
    
        let mut footer_html = String::from("<footer>");

    
        footer_html.push_str(&markdown_to_html(&footer, &ComrakOptions {
            render: comrak::ComrakRenderOptions {
                unsafe_: true,
                ..Default::default()
            },
            ..Default::default()
        }));
        footer_html.push_str("</footer>");

        return footer_html;
    }

    return String::new();
}

pub fn generate_head(pages_dir: &str) -> String {
    if Path::new(pages_dir).join("_head.html").exists() {
        let nav = fs::read_to_string(Path::new(pages_dir).join("_head.html"))
            .expect("Couldn't read head file");

        return nav;
    } 

    return String::new();
}

pub fn generate_nav(pages_dir: &str) -> String {
    if Path::new(pages_dir).join("_nav.html").exists() {
        let nav = fs::read_to_string(Path::new(pages_dir).join("_nav.html"))
            .expect("Couldn't read nav file");

        return nav;
    } else if Path::new(pages_dir).join("_nav.md").exists() {
        let nav = fs::read_to_string(Path::new(pages_dir).join("_nav.md"))
            .expect("Couldn't read nav file");
    
        let mut nav_html = String::from("<nav>");

    
        nav_html.push_str(&markdown_to_html(&nav, &ComrakOptions {
            render: comrak::ComrakRenderOptions {
                unsafe_: true,
                ..Default::default()
            },
            ..Default::default()
        }));
        nav_html.push_str("</nav>");

        return nav_html;
    }

    return String::new();
}

fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        fs::remove_file(entry?.path())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_head_html() {
        let mut metadata = Map::new();
        metadata.insert("title".to_string(), Value::String("Test".to_string()));
        metadata.insert("description".to_string(), Value::String("Test description".to_string()));
        metadata.insert("root_dir".to_string(), Value::String("/".to_string()));

        let head = generate_head_html(DEFAULT_HEAD, metadata);

        assert!(head.contains("<title>Test</title>"));
        assert!(head.contains("<meta name=\"description\" content=\"Test description\">"));
        assert!(head.contains("<link rel=\"stylesheet\" href=\"/style.css\">"));

        let mut metadata = Map::new();
        metadata.insert("title".to_string(), Value::String("Test".to_string()));
        metadata.insert("description".to_string(), Value::String("Test description".to_string()));
        metadata.insert("root_dir".to_string(), Value::String("/test/".to_string()));

        let custom_head = "<title>%title%</title><meta name=\"description\" content=\"%description%\"><link rel=\"stylesheet\" href=\"%root_dir%style.css\">";
        let head = generate_head_html(custom_head, metadata);

        assert!(head.contains("<title>Test</title>"));
        assert!(head.contains("<meta name=\"description\" content=\"Test description\">"));
        assert!(head.contains("<link rel=\"stylesheet\" href=\"/test/style.css\">"));
    }

}
