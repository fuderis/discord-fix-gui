#![allow(unused_imports)]
use std::fs::{ self, File };
use std::io::{ BufRead, BufReader };
use std::path::{ Path, PathBuf };
use std::process::Command;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=ui/styles/scss");
    println!("cargo:rerun-if-changed=ui/scripts/ts");

    // compile SCSS:
    compile_scss_dir("ui/styles/scss", "ui/styles/css")?;

    // prepare JS:
    prepare_js_dir("ui/scripts/jsm", "ui/scripts/js")?;
    
    // build tauri:
    tauri_build::build();

    Ok(())
}

/// Compiles SCSS files dir
fn compile_scss_dir<P: AsRef<Path>>(input_dir: P, output_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let input_dir = input_dir.as_ref();
    let output_dir = output_dir.as_ref();
    
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        // recursive dirs:
        if path.is_dir() {
            // skip dirs named 'modules':
            if path.file_name().map_or(false, |name| name == "modules") { continue; }
            
            // prepare output path:
            let rel = path.strip_prefix(input_dir)?;
            let out_subdir = output_dir.join(rel);

            compile_scss_dir(&path, &out_subdir)?;
        }
        
        // compile scss files:
        else if path.extension().map_or(false, |ext| ext == "scss") {
            // skip file with comment in first line:
            let mut reader = BufReader::new(File::open(&path)?);
            let mut first_line = String::new();

            reader.read_line(&mut first_line)?;
            let trimmed = first_line.trim_start();
            
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // prepare output path:
            let rel = path.strip_prefix(input_dir)?.with_extension("css");
            let out_path = output_dir.join(rel);

            compile_scss_file(path, out_path)?;
        }
    }

    Ok(())
}

/// Compiles SCSS file
fn compile_scss_file<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // compile into CSS:
    let css = grass::from_path(&input_path, &grass::Options::default())?;
    
    // write results to file:
    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::write(output_path, &css)?;
    
    Ok(())
}


/// Prepares JS files dir
fn prepare_js_dir<P: AsRef<Path>>(input_dir: P, output_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let input_dir = input_dir.as_ref();
    let output_dir = output_dir.as_ref();
    
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        // recursive dirs:
        if path.is_dir() {
            // skip dirs named 'modules':
            if path.file_name().map_or(false, |name| name == "modules") { continue; }
            
            // prepare output path:
            let rel = path.strip_prefix(input_dir)?;
            let out_subdir = output_dir.join(rel);

            prepare_js_dir(&path, &out_subdir)?;
        }
        
        // compile scss files:
        else if path.extension().map_or(false, |ext| ext == "js") {
            // skip file with comment in first line:
            let mut reader = BufReader::new(File::open(&path)?);
            let mut first_line = String::new();

            reader.read_line(&mut first_line)?;
            let trimmed = first_line.trim_start();
            
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // prepare output path:
            let rel = path.strip_prefix(input_dir)?.with_extension("js");
            let out_path = output_dir.join(rel);

            prepare_js_file(path, out_path)?;
        }
    }

    Ok(())
}

/// Prepares JS file
fn prepare_js_file<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // compile into CSS:
    let js = js_process_imports(&input_path)?;
    
    // write results to file:
    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::write(output_path, &js)?;
    
    Ok(())
}

// Recursive handling JS imports
fn js_process_imports<P: AsRef<Path>>(input_path: P) -> Result<String, Box<dyn std::error::Error>> {
    let input_path = input_path.as_ref();
    let base_path = input_path.parent().unwrap();
    let content = fs::read_to_string(input_path)?;

    let import_re = Regex::new(r#"(?m)import\s+['"]([^'"]+)['"]\s*;"#).unwrap();
    let mut result = String::new();
    let mut last_end = 0;

    for cap in import_re.captures_iter(&content) {
        let m = cap.get(0).unwrap();
        let import_path = cap.get(1).unwrap().as_str();
        let full_path = base_path.join(import_path);

        // Рекурсивно обрабатываем импортируемый файл
        let processed_import = js_process_imports(&full_path)?;

        result.push_str(&content[last_end..m.start()]);
        result.push_str(&processed_import);
        last_end = m.end();
    }
    result.push_str(&content[last_end..]);

    Ok(result)
}
