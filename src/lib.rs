use std::{fs, io};
use std::path::PathBuf;
use std::error::Error;
use std::env;
//use regex::Regex;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let dirs: Vec<PathBuf>;
    let current_files: Vec<PathBuf>;

    if let Ok((directories, files)) = list_files() {
        dirs = directories;
        current_files = files;
        println!("These are the files: {:#?}", current_files);
        for file_path in current_files {
            let contents = fs::read_to_string(&file_path)?;
            
            let results = if config.ignore_case {
                search_case_insensitive(&config.query, &contents)
            } else {
                search(&config.query, &contents)
            };

            println!("\n\nIn: {:?}", file_path);
            for (index, line) in results {
                println!("l.{}: {line}", index + 1);
            } 
        }        
    } else {
        ();
    }
    

    Ok(())
}


fn list_files() -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let (dirs, files): (Vec<PathBuf>, Vec<PathBuf>) = fs::read_dir(".")?
        .map(|res| res.map(|e| e.path()))
        .fold((Vec::new(), Vec::new()), |(mut dirs, mut files), res| {
            match res {
                Ok(entry) => {
                    if entry.is_dir() {
                        dirs.push(entry);
                    } else {
                        files.push(entry);
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
                _ => return Err("Unknown flag or parameter"),
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
