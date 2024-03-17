mod model;

use model::player::Player;

use std::{io, thread};
use std::future::Future;
use std::io::Write;
use reqwest::{Client, Request, Response, StatusCode};
use std::time::Duration;
use log::error;
use tokio::{join, try_join};
use tokio::sync::{mpsc, mpsc::Sender};

const URL: &str = "http://localhost:1337/xogamedev/";

#[tokio::main]
async fn main() {
    env_logger::init();
    //there was supposed to be an auth_key here... i guess so, instead i login every single time
    let mut player:Option<Player> = None;
    let client = reqwest::Client::new();

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("BRUH");

        match input.as_str().trim() {
            "login" => {player = login(client.clone()).await},
            "register" => register(client.clone()).await,
            "create new game" => create_new_game(client.clone(), player.clone()).await,
            "find game" => find_game(client.clone()).await,
            "join game" => join_game(client.clone(), player.clone()).await,
            _ => help()
        }
    };
}

async fn login(client: Client) -> Option<Player> {
    let mut username = String::new();
    let mut password = String::new();
    print!("Username: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut username).expect("Breh");
    print!("Password: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut password).expect("BRAH");
    username = username.trim().to_string();
    password = password.trim().to_string();

    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, username, password);

    match client.post(format!("{}{}", URL, "login"))
        .body(json_body.clone())
        .send()
        .await {
        Ok(response) => {
            if response.status() == StatusCode::OK {
                println!("Login successfully");
                return Some(Player{username, password})
            } else {
                println!("Login failed");
                error!("login {} {} {}", json_body, response.status(), response.text().await.unwrap());
                None
            }
        },
        Err(e) => {
            println!("Login failed");
            error!("login {} {}", json_body, e.to_string());
            None
        }
    }
}

async fn register(client: Client) {
    let mut username = String::new();
    let mut password = String::new();
    print!("Username: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut username).expect("Breh");
    print!("Password: ");
    io::stdout().flush().expect("bruh");
    io::stdin().read_line(&mut password).expect("BRAH");
    username = username.trim().to_string();
    password = password.trim().to_string();

    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, username, password);
    match client.post(format!("{}{}", URL, "register"))
        .body(json_body.clone())
        .send()
        .await {
        Ok(response) => {
            if (response.status()) == StatusCode::OK {
                println!("Register successfully");
            } else {
                println!("Register failed");
                error!("register {} {} {}", json_body, response.status(), response.text().await.unwrap());
            }
        },
        Err(e) => {
            println!("Register failed");
            error!("register {} {}", json_body, e)
        }
    }
}

async fn create_new_game(client: Client, player: Option<Player>) {
    if player.is_some() {
        let player = player.unwrap();
        let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
        let response = client.post(format!("{}{}", URL, "create_new_game")).body(json_body).send().await.unwrap();
        if response.status() == StatusCode::OK {
            let game_id = response.text().await.unwrap();
            game_loop(client, player, true, game_id.as_str()).await;
        } else {
            error!("Create new game {}{}", response.status(), response.text().await.unwrap());
            println!("Error while creating new game session, please try again");
        }
    } else {
        println!("Can't start game, please log in first");
    }
}

async fn find_game(client: Client) {
    println!("List of games: \n{}", client.get(format!("{}{}", URL, "get_session")).send().await.unwrap().text().await.unwrap());
}

async fn join_game(client: Client, player: Option<Player>) {
    if player.is_some() {
        let player = player.unwrap();
        let mut game_id = String::new();
        io::stdin().read_line(&mut game_id).expect("Bla");
        let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
        let response = client.post(format!("{}{}{}", URL, "join_session/", game_id)).body(json_body).send().await
            .unwrap();
        if response.status() == StatusCode::OK {
            game_id = response.text().await.unwrap();
            game_loop(client, player,false, game_id.as_str()).await;
        } else {
            error!("Join game {} {}", response.status(), response.text().await.unwrap());
            println!("Error while joining game session, please try to join different game")
        }
    } else {
        println!("Can't start game, please log in first");
    }
}

async fn game_loop(client: Client, player: Player, wait_for_player2: bool, game_id: &str) {

    if wait_for_player2 {
        if !wait_for_player_two(client.clone(), player.clone(), game_id).await {
            println!("Opponent surrendered");
        }
        println!("You are player 1");
    } else {
        println!("You are player 2");
    }
    loop {
        if !make_a_move(client.clone(), player.clone(), game_id).await && !wait_for_player_two(client.clone(), player.clone(), game_id).await {
            break
        }
    }
    println!("Game ended");
}

async fn print_loading_screen() {
    let arr = ["[=    ]", "[==    ]", "[===  ]", "[==== ]", "[=====]"];
    for i in arr {
        print!("{}", i);
        io::stdout().flush().unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_player_two(client: Client, player: Player, game_id: &str) -> bool {
    let (tx, mut rx) = mpsc::channel(10);
    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
    tokio::spawn(wait_for_move(client.clone(), tx.clone(), json_body, game_id.to_string()));
    tokio::spawn(wait_for_surrender(client.clone(), tx.clone(), player, game_id.to_string()));
    loop {
        let rx_value = rx.try_recv();
        if let Ok(ref integer) = rx_value {
            if integer == &0.to_string() {
                return false;
            }
        }
        match rx_value {
            Ok(text) => {
                println!("{}", text);
                return true;
            },
            Err(_) => print_loading_screen().await
        }
    }
}
async fn wait_for_move(client: Client, tx: Sender<String>, json_body: String, game_id: String) {
    tx.send(client.post(format!("{}{}", URL, game_id)).body(json_body).send().await.unwrap().status().to_string()).await.expect("bruh");
}

async fn wait_for_surrender(client: Client, tx: Sender<String>, player: Player, game_id: String) {
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).expect("bla");
    match user_input.trim().parse::<i32>() {
        Ok(0) => {
            surrender(client, player, game_id).await;
            tx.send(0.to_string()).await.unwrap();
        }
        _ => help_surrender()
    }
}

async fn wait_for_first_move(client: Client, player: Player, game_id: &str) -> bool {
    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
    if let r = client.post(format!("{}{}", URL, game_id)).body(json_body).send().await.unwrap().text().await.unwrap() {
        //TODO check if status code = 2xx and extract first player to move from r
        return true;
    }
    false
}

async fn make_a_move(client: Client, player: Player, game_id: &str) -> bool {
    loop {
        let mut player_input = String::new();
        io::stdin().read_line(&mut player_input).expect("bruh");

        let player_input = player_input.trim().parse::<i32>();

        match player_input {
            Ok(0) => {
                surrender(client, player, game_id.to_string()).await;
                return false;
            },
            Ok(1..=9) => {
                let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
                return if let Ok(response) = client.post(format!("{}{}{}{}", URL, game_id, "/make_a_move?move=", player_input.unwrap()))
                    .body(json_body)
                    .send()
                    .await {
                    let result = response.text().await.unwrap();
                    if result.len() > 1 {
                        println!("{}", result);
                        true
                    } else {
                        let value = result.parse::<i32>().unwrap();
                        if value == 1 {
                            println!("Player 1 has won the game");
                        } else if value == 2 {
                            println!("Player 2 has won the game");
                        } else {
                            println!("Draw, no one won the game");
                        }
                        false
                    }
                } else {
                    true
                }
            },
            _ => { help_xo() }
        }
    }
}

async fn surrender(client: Client, player: Player, game_id: String) {
    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
    match client.post(format!("{}{}{}", URL, game_id, "/surrender"))
        .body(json_body)
        .send()
        .await {
        Ok(_) => println!("OK"),
        Err(_) => println!("Server error")
    }
}

// async fn check_for_server_status(r: Response) {
//     match r {
//         Ok(0) => println!("Opponent surrendered"),
//         Ok(1) => println!("Opponent won"),
//         Ok(2) => println!("You won")
//     }
// }

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

fn help_surrender() {
    println!("Command: \n\
    0 - surrender\n")
}