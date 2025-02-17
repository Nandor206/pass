use clap::{value_parser, Arg, Command};
use rand::seq::IteratorRandom;
use std::fs::{self, File};
use std::process::Command as ProcessCommand;
use dirs;
use clipboard::{self, ClipboardContext, ClipboardProvider};
use std::io::*;

// Gets args
fn get_args() -> Command {
    Command::new("pass")
        .version("1.0")
        .about(
            "Password maker, with default length of 15. Uses random characters and stores them in a file with their names"
        )
        .author("Nandor206")
        .arg(
            Arg::new("length")
                .short('l')
                .long("length")
                .value_parser(value_parser!(u32))
                .default_value("15")
                .help("Password length")
        )
        .arg(
            Arg::new("specials")
                .short('s')
                .long("special")
                .action(clap::ArgAction::SetFalse)
                .help("Include special characters")
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_parser(value_parser!(String))
                .default_value("")
                .help("Name of the password")
        )
}

// Generates password
fn generate_password(length: u32, include_specials: bool) -> String {
    let mut chars = String::new();
    chars.push_str(LOWERCASE);
    chars.push_str(UPPERCASE);
    chars.push_str(DIGITS);
    if include_specials {
        chars.push_str(SPECIALS);
    }

    let mut rng = rand::thread_rng();
    let mut password = String::new();
    for _ in 0..length {
        if let Some(c) = chars.chars().choose(&mut rng) {
            password.push(c);
        }
    }
    password
}

// Check if path exists, if not, create
fn check_path(path: &std::path::Path) {
    if !path.exists() {
        std::fs::create_dir_all(path).expect("Failed to create directory");
    }
    let file_path = path.join("passes.txt");
    if !file_path.exists() {
        File::create(&file_path).expect("Failed to create file");
    }
}

// Writeing the passwords to file
fn file(password: &str, name: &str, path: &std::path::Path) {
    let file_path = path.join("passes.txt");
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&file_path)
        .expect("Failed to open file");
    writeln!(file, "{}: {}", name, password).expect("Failed to write to file");
}

// fzf implementation for copying passwords
fn fzf_files(path: &std::path::Path) -> String {
    let file_path = path.join("passes.txt");
    let file_contents = std::fs::read_to_string(&file_path).expect("Failed to read file");
    if file_contents.is_empty() {
        println!("No passwords found");
        return String::new();
    } else {
        let mut child = ProcessCommand::new("fzf")
            .arg("--height=40%")
            .arg("--border")
            .arg("--prompt=Select a password: ")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn fzf");

        child.stdin.as_mut().expect("Failed to open stdin")
            .write_all(file_contents.as_bytes())
            .expect("Failed to write to stdin");

        let output = child.wait_with_output()
            .expect("Failed to read stdout");

        if output.status.success() {
            let selected_password = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let mut clipboard: ClipboardContext = ClipboardProvider::new().expect("Failed to access clipboard");
            clipboard.set_contents(selected_password.clone()).expect("Failed to copy to clipboard");
            println!("Password copied to clipboard");
            selected_password
        } else {
            println!("No selection made or fzf failed");
            String::new()
        }
    }
}

//Constant stuff
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &str = "0123456789";
const SPECIALS: &str = "!@#$%^&*()-_=+[{]};:'\",.<>/?";

// Main
fn main() {
    let matches = get_args().get_matches();

    let length = matches.get_one::<u32>("length").cloned().unwrap_or(15);
    let include_specials = !matches.get_flag("specials");
    let name = matches.get_one::<String>("name").cloned().unwrap_or_default();

    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let path = home_dir.join(".config").join("pass");

    check_path(&path);

    if name.is_empty() {
        let output_fzf = fzf_files(&path);
        if !output_fzf.is_empty() {
            let mut clipboard: ClipboardContext = ClipboardProvider::new().expect("Failed to access clipboard");
            let parts: Vec<&str> = output_fzf.splitn(2, ": ").collect();
            if parts.len() == 2 {
                let name = parts[0];
                let password = parts[1];
                println!("Selected app: {}", name);
                println!("The app's password: {}", password);
                clipboard.set_contents(password.to_string()).expect("Failed to copy to clipboard");
                println!("Password copied to clipboard");
            } else {
                println!("Invalid password format");
            }
        } else {
            println!("No password selected or there are no passwords in the database");
        }
    } else {
        let password = generate_password(length, include_specials);
        println!("Generated password: {}", password);
        file(&password, &name, &path);
        println!("Saved password to {}", path.join("passes.txt").display());
    }
}
