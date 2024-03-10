mod model;

use model::player::Player;

use std::{io, thread};
use std::future::Future;
use std::io::Write;
use reqwest::{Client, Request, StatusCode};
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
            //TODO check 404
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
        if let result = client.get(format!("{}{}", URL, "create_new_game")).body(json_body).send().await.unwrap().text().await.unwrap() {
            //TODO check r status, if it's code 2xx and extract game id also
            //TODO get into game loop
            let game_id = "";
            game_loop(client, player, true, game_id).await;
        }

    } else {
        println!("Can't start game, if you have already logged in, please try again");
    }
}

async fn find_game(client: Client) {
    println!("List of games: \n{}", client.get(format!("{}{}", URL, "find_game")).send().await.unwrap().text().await.unwrap());
}

async fn join_game(client: Client, player: Option<Player>) {
    if player.is_some() {
        let player = player.unwrap();
        let mut game_id = String::new();
        io::stdin().read_line(&mut game_id).expect("Bla");
        let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
        if let r = client.post(format!("{}{}{}", URL, "join_game/", game_id)).body(json_body).send().await
            .unwrap().text().await.unwrap() {
            //TODO check r status, if it's code 2xx
            //TODO get into game loop
            game_loop(client, player,false, game_id.as_str()).await;
        }
    } else {
        println!("Can't start game, if you have already logged in, please try to join different game");
    }
}

async fn game_loop(client: Client, player: Player, wait_for_player2: bool, game_id: &str) {
    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password).as_str();

    if wait_for_player2 {
        wait_for_player_two(client, player, game_id).await;
    }


}

async fn print_loading_screen() {
    let arr = ["[=    ]", "[==    ]", "[===  ]", "[==== ]", "[=====]"];
    for i in arr {
        print!("{}", i);
        io::stdout().flush().unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_player_two(client: Client, player: Player, game_id: &str) {
    let (tx, mut rx) = mpsc::channel(10);
    let json_body = format!(r#"{{"username": "{}", "password": "{}" }}"#, player.username, player.password);
    let a = tokio::spawn(a(client.clone(), tx.clone(), json_body, game_id.to_string()));
    let b = tokio::spawn(b(client.clone(), tx.clone(), player, game_id.to_string()));
    loop {
        match rx.try_recv() {
            Ok(_) => break,
            Err(_) => print_loading_screen().await
        }
    }
}
async fn a(client: Client, tx: Sender<String>, json_body: String, game_id: String) {
    tx.send(client.post(format!("{}{}", URL, game_id)).body(json_body).send().await.unwrap().text().await.unwrap()).await.expect("bla");
}

async fn b(client: Client, tx: Sender<String>, player: Player, game_id: String) {
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).expect("bla");
    match user_input.trim().parse::<i32>() {
        Ok(0) => { surrender(client, player, game_id).await }
        _ => help_xo()
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

async fn make_a_move(client: Client, player: Player, game_id: &str) {
    let mut player_input = String::new();
    io::stdin().read_line(&mut player_input).expect("bruh");

    match player_input.trim().parse::<i32>() {
        Ok(0) => { surrender(client, player, game_id.to_string()).await;},
        Ok(1..=9) => {

        },
        _ => {help_xo()}
    }
}

async fn surrender(client: Client, player: Player, game_id: String) {

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