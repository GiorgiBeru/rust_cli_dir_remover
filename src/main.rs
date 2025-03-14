use std::path::{Path};
use std::fs;
use std::io::{self, BufRead};
use walkdir::WalkDir;


use std::env;

#[derive(Debug)]
struct Args {
    cleanup_file: String,
    dry: bool,
}

struct DeletionStats {
    bytes_freed: u64,
    directories_removed: usize,
}

impl Args {
    fn create_args(args: Vec<String>) -> Self {
        let mut cleanup_file = ".cleanup".to_string(); 
        let mut dry = false;

        let mut iter = args.iter().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-c" | "--cleanup-file" => {
                    if let Some(value) = iter.next() {
                        cleanup_file = value.clone();
                    }
                }
                "-d" | "--dry" => {
                    dry = true;
                }
                _ => {
                    panic!("Invalid flags provided: {}", arg);
                }
            }
        }

        Args { cleanup_file, dry }
    }
}


fn main() {
    let env_args: Vec<String> = env::args().collect();
    let args = Args::create_args(env_args);

    let project_root = Path::new(".").canonicalize().expect("Could not determine root directory");
    
    let mut stats = DeletionStats {
        bytes_freed: 0,
        directories_removed: 0
    };

    let file_path_to_read = Path::join(&project_root, args.cleanup_file);
    if !file_path_to_read.exists() {
        panic!("Invalid file source provided");
    }
    

    let dir_vec = directories_to_remove(&file_path_to_read).expect("Could not determine directories to remove");

    if dir_vec.is_empty() {
        println!("No directories found to remove");
        return;
    } 
    

    let mut dirs_to_remove_path = Vec::new();
    for directory in dir_vec {
        let dir_to_rm_path = Path::join(&project_root, &directory);
        dirs_to_remove_path.push(Path::join(&project_root, &directory));
        
        if dir_to_rm_path.exists() && dir_to_rm_path.is_dir() {
            let size = calculate_dir_size(&dir_to_rm_path).expect("Indeterminable size");
            stats.bytes_freed += size;
            stats.directories_removed += 1;
            if !args.dry {
                let _ = fs::remove_dir_all(&dir_to_rm_path).expect("Error removing");
            }
        }
    }

    if args.dry {
        println!("Space will be freed: {} Bytes", stats.bytes_freed);
        println!("Directories will be removed {}", stats.directories_removed);
        println!("Dir list: {:?}", dirs_to_remove_path);
    } else {
        println!("Space freed: {} Bytes", stats.bytes_freed);
        println!("Directories removed {}", stats.directories_removed);
        println!("Dir list: {:?}", dirs_to_remove_path);
    }
}

fn directories_to_remove(path: &Path) -> Result<Vec<String>, io::Error>{
    let open_file = fs::File::open(path)?;
    let read = io::BufReader::new(open_file);

    let mut directories_to_remove: Vec<String> = Vec::new();

    for line in read.lines() {
        let line = line?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            directories_to_remove.push(trimmed.to_string());
        }
    }
    Ok(directories_to_remove)
}


fn calculate_dir_size(path: &Path) -> Result<u64, io::Error> {
    let mut total_size = 0;
    
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }
    
    Ok(total_size)
}
//TODO: add fn for git tracking detection
// fn -> add git tracks directory to remove check -> if .git file in root directory and target directory not in .gitignore then tracks
// fn is_git_repo(path: &Path) -> bool {
//     match Repository::open(path) {
//         Ok(_) => true,
//         Err(_) => false,
//     }
// }

//TODO: add flag verbose with dbg! macros for debugging