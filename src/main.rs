use std::{io, process};
use charybdis::db::Db;

struct Command {
    name: String,
    args: Vec<String>
}

struct CommandResult {
    success: bool,
    body: String
}

struct AppState {
    db: Db,
    is_running: bool
}

fn main() {
    let mut app_state = AppState {
        db: Db::new(),
        is_running: true
    };

    app_state.db.init().expect("Error initializing database...");


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
            let result: Option<Vec<u8>> = state.db.get(&command.args[0]);
            match result {
                None => CommandResult{ success: true, body: String::from("<nil>")},
                Some(value) => CommandResult{ success: true, body: String::from_utf8(value).unwrap()}
            }
        },
        "SET" => {
            if command.args.len() < 2 {
                return CommandResult{ success: false, body: String::from("Invalid number of arguments")};
            }
            let result= state.db.put(command.args[0].clone(), command.args[1].clone());
            match result {
                Err(e) => CommandResult{ success: false, body: format!("{}", e)},
                Ok(_) => CommandResult{ success: true, body: String::from("<OK>")}
            }
        },
        "DEL" => {
            if command.args.len() < 1 {
                return CommandResult{ success: false, body: String::from("Invalid number of arguments")};
            }
            let result= state.db.delete(command.args[0].clone());
            match result {
                Err(e) => CommandResult{ success: false, body: format!("{}", e)},
                Ok(_) => CommandResult{ success: true, body: String::from("<OK>")}
            }
        }
        _ => CommandResult{ success: false, body: String::from("Command not found") },
    }
}