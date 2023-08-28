use clap::{Parser, Subcommand};
use std::path::Path;
use crate::compiler::{
    get_files,
    generate_head,
    generate_nav,
    generate_build_dir, 
    generate_markdown_files, 
    generate_footer, 
    MarkdownFile
};

mod compiler;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// Builds the site
    Build {
        /// Sets the static folder location
        #[arg(short, long, value_parser = directory_validator)]
        source_dir: String,

        /// Sets the build folder location
        #[arg(short, long, default_value_t = String::from("build"))]
        build_dir: String,

        /// Sets root directory
        #[arg(short, long, default_value_t = String::from("/"))]
        root_dir: String,
    } 
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Build { source_dir, build_dir, root_dir } => {
            println!("Started build step...\n");

            let markdown_files: Vec<MarkdownFile> = get_files(&source_dir);
            let footer: String = generate_footer(&source_dir);
            let nav: String = generate_nav(&source_dir);
            let head: String = generate_head(&source_dir);
            let root_dir: &str = &root_dir.trim_end_matches("/");

            generate_build_dir(&build_dir, &source_dir)
                .expect("Could not generate build directory");
            
            println!("\nGenerating markdown files...");
            generate_markdown_files(&markdown_files, &build_dir, &source_dir, &root_dir, &head, &nav, &footer);

            println!("Successfully built {} markdown files", markdown_files.len());
        }
    }

}

fn directory_validator(s: &str) -> Result<String, String> {
    if Path::new(&s).is_dir() {
        Ok(s.to_string())
    } else {
        Err(format!("{} is not a directory", s))
    }
}
