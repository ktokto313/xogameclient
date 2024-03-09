mod model;

use model::player::Player;

use std::io;
use std::io::Write;
use reqwest::{Client, Request};

#[tokio::main]
async fn main() {

    //there was supposed to be an auth_key here... i guess so, instead i login every single time
    let mut player:Option<Player> = None;
    let client = reqwest::Client::new();
    let url = "localhost:1337/xogamedev/";

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("BRUH");

        match input.as_str().trim() {
            "login" => {player = login(url, client.clone()).await},
            "register" => register(url, client.clone()).await,
            "create new game" => create_new_game(url, player.clone(), client.clone()).await,
            "find game" => find_game(url, client.clone()).await,
            "join game" => join_game(url, player.clone(), client.clone()).await,
            _ => help()
        }
    };
}

async fn login(url: &str, client: Client) -> Option<Player> {
    let mut username = String::new();
    let mut password = String::new();
    print!("Username: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut username).expect("Breh");
    print!("Password: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut password).expect("BRAH");

    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, username, password);

    match client.post(url.to_string() + "login")
        .body(json_body)
        .send()
        .await {
        Ok(_) => {
            println!("Login successfully");
            Some(Player{username, password})
        },
        Err(_) => {
            println!("Login failed");
            None
        }
    }
}

async fn register(url: String, client: Client) {
    let mut username = String::new();
    let mut password = String::new();
    print!("Username: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut username).expect("Breh");
    print!("Password: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut password).expect("BRAH");

    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, username, password);

    match client.post(url.to_string() + "register")
        .body(json_body)
        .send()
        .await {
        Ok(_) => {
            println!("Register successfully");
        },
        Err(_) => {
            println!("Register failed");
        }
    }
}

async fn create_new_game(url: String, player: Option<Player>, client: Client) {
    if player.is_some() {
        client.get(url.to_string() + "create_new_game").send().await.unwrap().text().await.unwrap();
        //TODO get into game loop
    } else {
        println!("Can't start game, please login first");
    }
}

async fn find_game(url:String, client: Client) {
    println!("List of games: \n{}", client.get(url.to_string() + "find_game").send().await.unwrap().text().await.unwrap());
}

async fn join_game(url: String, player: Option<Player>, client: Client) {
    if player.is_some() {
        let mut game_id = String::new();
        io::stdin().read_line(&mut game_id).expect("Bla");
        client.get(format!("{}{}", url.to_string() + "join_game/", game_id)).send().await.unwrap().text().await.unwrap();
        //TODO get into game loop

    } else {
        println!("Can't start game, if you have already logged in, please try to join different game");
    }
}

async fn game_loop() {
    loop {
        let mut player_input = String::new();
        io::stdin().read_line(&mut player_input).expect("bruh");

        match player_input.trim().parse::<i32>() {
            Ok(0) => {},
            Ok(1..=9) => {},
            _ => {help_xo()}
        }
    }
}

fn help() {
    println!("Command: \n\
    login - log into a created account\n\
    register - register a new account\n\
    create_new_game - create a new xo game session\n\
    find_game - list all game session waiting for player\n\
    join_game - join game session")
}

fn help_xo() {
    println!("Command: \n\
    1-9 - make a move\n\
    0 - surrender\n")
}