use gray_matter::Matter;
use gray_matter::engine::YAML;
use walkdir::WalkDir;
use serde::Deserialize;
use std::{path::Path, io::Result, fs::{self, create_dir_all}};
use comrak::{markdown_to_html, ComrakOptions};

macro_rules! HEAD {
    () => {
        "<head>\
            <meta charset=\"utf-8\">\
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
            <link rel=\"icon\" href=\"/favicon.png\">\
            <link rel=\"stylesheet\" href=\"/style.css\">\
            <title>{}</title>\
            <meta name=\"description\" content=\"{}\">\
        </head>"
    };
}


pub struct MarkdownFile {
    path: String,
    body: String
}

#[derive(Deserialize, Default)]
struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,

    pub nav: Option<bool>,
    pub footer: Option<bool>
}


pub fn get_files(source_dir: &str) -> Vec<MarkdownFile> {
    let mut markdown_files: Vec<MarkdownFile> = Vec::new();
    

    let walker = WalkDir::new(source_dir).into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().unwrap().ends_with(".md"))
        .filter(|e| !e.file_name().to_str().unwrap().starts_with("_"))
        .filter(|e| e.metadata().expect("Couldn't fetch metadata").is_file());


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


// TODO: Merge the static folder and the pages folder

pub fn generate_build_dir(build_dir: &str, source_dir: &str) -> Result<()> {
    println!("Creating build folder...");

    let _ = fs::remove_dir_all(build_dir);

    fs::create_dir_all(build_dir)
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

pub fn generate_markdown_files(markdown_files: &Vec<MarkdownFile>, build_dir: &str, source_dir: &str, nav: &str, footer: &str) {
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

        let metadata: FrontMatter = match result.data {
            Some(parsed) => {
                let front_matter: FrontMatter = match parsed.deserialize() {
                    Ok(fm) => fm,
                    Err(_) => FrontMatter::default()
                };
            
                front_matter
            }
            None => FrontMatter::default()
        };

        let mut html = String::from("<html>");
        html.push_str(&format!(HEAD!(), metadata.title.unwrap_or("Title".to_owned()), metadata.description.unwrap_or("Description".to_owned())));
        html.push_str("<body><main>");

        // Unwrap metadata.nav or use default value
        // from FrontMatter struct
        if nav != "" && metadata.nav.unwrap_or(true) {
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
        
        if footer != "" && metadata.footer.unwrap_or(true) {
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

pub fn generate_nav(pages_dir: &str) -> String {
   let dir = fs::read_dir(pages_dir).expect("Couldn't read pages directory");
    
    let nav = dir.filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_str().unwrap() == "_nav.md")
        .map(|entry| fs::read_to_string(entry.path()).unwrap())
        .collect::<Vec<String>>();

    if nav.len() > 0 {
        let mut nav_html = String::from("<nav>");
        nav_html.push_str(&markdown_to_html(&nav[0], &ComrakOptions {
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
