use clap::{value_parser, Arg, Command};
use std::fs::{self, File};
use std::io::Write;
use std::process::Command as ProcessCommand;
use dirs;
use clipboard::{self, ClipboardContext, ClipboardProvider};

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
        password.push(chars.chars().choose(&mut rng).unwrap());
    }
    password
}

// Check if path exists, if not, create
fn check_path() {
    let path = std::path::Path::new(PATH);
    if !path.exists() {
        std::fs::create_dir_all(path).expect("Failed to create directory");
    }
    let file_path = format!("{}passes.txt", PATH);
    if !std::path::Path::new(&file_path).exists() {
        File::create(&file_path).expect("Failed to create file");
    }
}

// Writeing the passwords to file
fn file(password: String, name: String) {
    let file_path = format!("{}passes.txt", PATH);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&file_path)
        .expect("Failed to open file");
    writeln!(file, "{}: {}", name, password).expect("Failed to write to file");
}

// fzf implementation for copying passwords
fn fzf_files() -> String {
    let file_contents = std::fs::read_to_string(format!("{}passes.txt", PATH)).expect("Failed to read file");
    if file_contents.is_empty() {
        println!("No passwords found");
        return String::new();
    } else {
        let output = ProcessCommand::new("fzf")
            .arg("--height=40%")
            .arg("--border")
            .arg("--prompt=Select a password: ")
            .arg("--preview=cat {}")
            .output()
            .expect("Failed to execute fzf");

        if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
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
const HOME_DIR: &str = dirs::home_dir();
const PATH: &str = format!("{}/.config/pass/", HOME_DIR).as_str();

// Main
fn main() {
    let matches = get_args().get_matches();

    let length = matches.value_of("length").unwrap().parse().unwrap();
    let include_specials = matches.is_present("specials");
    let name = matches.value_of("name").unwrap_or_default().to_string();

    check_path();

    if name.is_empty() {
        let output_fzf = fzf_files();
        if !output_fzf.is_empty() {
            let mut clipboard: ClipboardContext = ClipboardProvider::new().expect("Failed to access clipboard");
            let password = output_fzf.split(": ").last().unwrap().to_string();
            let name = output_fzf.split(": ").first().unwrap().to_string();
            print!("Selected app: {}", name);
            println!("The app's password: {}", password);
            clipboard.set_contents(password.to_string()).expect("Failed to copy to clipboard");
            println!("Password copied to clipboard");
        }
        else {
            println!("No password selected or there are no passwords in the database");
        }
    } else {
        let password = generate_password(length, include_specials);
        println!("Generated password: {}", password);
        file(password, name);
        println!("Saved password to {}passes.txt", PATH);
    }
}
