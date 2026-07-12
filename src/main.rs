use std::{io, process};
use std::collections::HashMap;

struct Command {
    name: String,
    args: Vec<String>
}

struct CommandResult {
    success: bool,
    body: String
}

struct AppState {
    data: HashMap<String, String>,
    is_running: bool
}

fn main() {
    let mut app_state = AppState {
        data: HashMap::new(),
        is_running: true
    };


    while app_state.is_running {
        let mut word = String::new();
        println!("\n> ");
        io::stdin()
            .read_line(&mut word)
            .expect("Failed to read line");

        word = word.trim().to_string();

        match parse(&word) {
            None => {
                println!("Invalid command");
            }
            Some(command) => {
                let result = dispatch(&command, &mut app_state);
                println!("< ({}) {}", result.success, result.body);
            }
        }
    }
    process::exit(0);
}

fn parse(word: &str) -> Option<Command> {
    if word.is_empty() {
        return None;
    }
    let tokens: Vec<&str> = word.split(" ").collect();
    if tokens.len() == 0 || tokens.len() == 1 && tokens[0].is_empty(){
        return None
    }
    let mut args = vec![];
    if tokens.len() > 1 {
        for token in &tokens[1..] {
            args.push(token.to_string())
        }
    }

    Some(Command {
        name: tokens[0].to_string(),
        args
    })
}

fn dispatch(command: &Command, state: &mut AppState) -> CommandResult {
    match command.name.to_uppercase().as_str() {
        "EXIT" => {
            state.is_running = false;
            CommandResult{ success: true, body: String::new()}
        },
        "GET" => {
            if command.args.len() < 1 {
                return CommandResult{ success: false, body: String::from("Invalid number of arguments")};
            }
            match state.data.get(&command.args[0].to_string()) {
                None => CommandResult{ success: true, body: String::from("<nil>")},
                Some(value) => CommandResult{ success: true, body: String::from(value)}
            }
        },
        "SET" => {
            if command.args.len() < 2 {
                return CommandResult{ success: false, body: String::from("Invalid number of arguments")};
            }
            match state.data.insert(command.args[0].to_string(), command.args[1].to_string()) {
                None => CommandResult{ success: true, body: String::from("<INSERTED>")},
                Some(_) => CommandResult{ success: true, body: String::from("<UPDATED>")}
            }
        },
        "DEL" => {
            if command.args.len() < 1 {
                return CommandResult{ success: false, body: String::from("Invalid number of arguments")};
            }
            match state.data.remove(&command.args[0].to_string()) {
                None => CommandResult{ success: true, body: String::from("<nil>")},
                Some(_) => CommandResult{ success: true, body: String::from("<REMOVED>")}
            }
        }
        _ => CommandResult{ success: false, body: String::from("Command not found") },
    }
}