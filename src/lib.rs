use std::{fs, io};
use std::path::PathBuf;
use std::error::Error;
use std::env;
//use regex::Regex;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {

    let mut dir_queue: Vec<PathBuf> = vec![];
    let mut path: PathBuf = PathBuf::from(&config.file_path);

    loop {
        if let Ok((directories, files)) = list_files(path) {
            let dirs: Vec<PathBuf> = if !config.hidden { directories.iter().filter(|x| find_hidden_files(x)).cloned().collect() }
                                                      else { directories };
            let current_files: Vec<PathBuf> = if !config.hidden { files.iter().filter(|x| find_hidden_files(x)).cloned().collect() }
                                              else { files };
            
            for file_path in current_files {
                let contents = fs::read_to_string(&file_path)?;
                
                let results = if config.ignore_case {
                    search_case_insensitive(&config.query, &contents)
                } else {
                    search(&config.query, &contents)
                };
                if results.len() > 0 {
                    println!("\nIn: {:?}", file_path);

                    if config.count {
                        println!("Found: {} occurrences", results.len());
                        continue
                    }

                    for (index, line) in results {
                        println!("l.{:03}: {}", index + 1, line.trim());
                        ();
                    }
                } 
            }
            
            dir_queue.extend(dirs.iter().map(|p| p.to_owned()));

            if dir_queue.len() < 1 || !config.recursive {
                break;
            } else {
                path = dir_queue.pop().expect("This is the test");
            }


        } else {
            break ();
        }
    }

    Ok(())
}


fn find_hidden_files(input: &PathBuf) -> bool {
    let string_path: String = input.to_string_lossy().into_owned();
    let parts: Vec<&str> = string_path.split('/').collect();

    if let Some(last_part) = parts.last() {
        if !last_part.is_empty() && last_part.starts_with('.') {
            false
        } else {
            true
        }
    } else {
        false
    }
}


fn list_files(path: PathBuf) -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (dirs, files): (Vec<PathBuf>, Vec<PathBuf>) = fs::read_dir(path)?
        .map(|res| res.map(|e| e.path()))
        .fold((Vec::new(), Vec::new()), |(mut dirs, mut files), res| {
            match res {
                Ok(entry) => {
                        
                        if entry.is_dir() {
                            dirs.push(entry);
                        } else {
                            if is_utf8_file(&entry) {
                                files.push(entry);
                            }
                        } 
                }
                Err(_) => {} // Ignore errors for simplicity, handle them as needed
            }
            (dirs, files)
        });

    Ok((dirs, files))
}


#[derive(Default)]
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
    pub recursive: bool,
    pub hidden: bool,
    pub count: bool,
}


impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }
        
        let mut args_iter = args.iter();
        
        args_iter.next();
        
        let query = args_iter.next().ok_or("Missing query argument")?.to_string();
        let file_path = args_iter.next().ok_or("Missing file path argument")?.to_string();

        let mut config = Config::default();

        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "-i" => config.ignore_case = true,
                "-r" => config.recursive = true,
                "-a" => config.hidden = true,
                "-c" => config.count = true,
                _    => return Err("Unknown flag or parameter"),
            }
        }

        let ignore_case = env::var("IGNORE_CASE").is_ok();
     
        config.query = query;
        config.file_path = file_path;
        config.ignore_case = ignore_case;

        Ok(config)
    }
}


pub fn search<'a>(query: &str, contents: &'a str) -> Vec<(usize, &'a str)> {
    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(query))
        .collect()
}


pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<(usize, &'a str)> {
    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| line.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}

fn is_utf8_file(path: &PathBuf) -> bool {
    match fs::read_to_string(path) {
        Ok(_) => true,  // File is successfully read as UTF-8
        Err(_) => false, // File is not UTF-8 encoded or an error occurred
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec![(1, "safe, fast, productive.")], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec![(0, "Rust:"), (3, "Trust me.")],
            search_case_insensitive(query, contents)
        );
    }
}


// TODO: Support regular expressions
// TODO: Turn this project into a installable application
