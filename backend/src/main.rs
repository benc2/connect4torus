#[macro_use]
extern crate rocket;
// use common::{board::Board, GameData, Player};
use mysql::prelude::Queryable;
use mysql::{params, Pool};
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::http::{Cookie, CookieJar};
use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::Request;
use rocket::State;
use std::fs;
use std::path::PathBuf;
use uiv2::Player; // uiv2 is now a lib which might be a bit of a hack
                  // perhaps define GameData in common, then wrap it in ConnectGame in ui and implement component on that
                  // in backend we can use GameData directly since we don't need to impl any traits on it
                  // but wrapper classes are annoying and ugly
use uiv2::connectgame::GameData;
use uiv2::gamelist::{GameList, GameLobby};
use uiv2::IdType;

#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> Result<NamedFile, NotFound<String>> {
    if cookies.get("session_id").is_none() {
        cookies.add(Cookie::build("session_id", rand::random::<IdType>().to_string()).finish())
    }
    // get_index().await
    NamedFile::open("../uiv2/dist/index.html")
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("404: Get outta here!\n {}", req.uri())
}

#[get("/<filename>", rank = 0)]
async fn getfile(filename: &str, cookies: &CookieJar<'_>) -> Result<NamedFile, NotFound<String>> {
    let mut filepath = PathBuf::from("../uiv2/dist/");
    filepath.push(filename);
    match NamedFile::open(filepath).await {
        Ok(f) => Ok(f),
        Err(_) => index(cookies).await,
    }
    // .map_err(|e| NotFound(e.to_string()))
}

#[get("/<_path..>", rank = 1)]
async fn redirect_ui(
    _path: PathBuf,
    cookies: &CookieJar<'_>,
) -> Result<NamedFile, NotFound<String>> {
    index(cookies).await
}

async fn get_lobbies(filter: &str, pool: &State<Pool>) -> Result<String, String> {
    let query = &format!(
        "SELECT game_id, player1_id, player2_id, game_name, game_started from gamelist where {}",
        filter
    );

    let mut conn = pool.inner().get_conn().expect("failed to connect to db");
    let games = conn
        .query_map(
            query,
            |(game_id, player1_id, player2_id, game_name, game_started)| GameLobby {
                game_id,
                player1_id,
                player2_id,
                game_name,
                game_started,
            },
        )
        .map_err(|_| "failed to get games from database")?;

    let gamelist = GameList { games: games };

    serde_json::to_string(&gamelist).map_err(|_| "serializing failed".to_owned())
}

#[get("/gamelistdata")]
async fn getgamelist(pool: &State<Pool>) -> Result<String, String> {
    // let query = "SELECT game_id, player1_id, player2_id, game_name from gamelist where player1_id is null or player2_id is null";

    // let mut conn = pool.inner().get_conn().expect("failed to connect to db");
    // let games = conn
    //     .query_map(query, |(game_id, player1_id, player2_id, game_name)| {
    //         GameLobby {
    //             game_id,
    //             player1_id,
    //             player2_id,
    //             game_name,
    //         }
    //     })
    //     .map_err(|_| "failed to get games from database")?;

    // let gamelist = GameList { games: games };

    // serde_json::to_string(&gamelist).map_err(|_| "serializing failed".to_owned())
    let filter = "player1_id is null or player2_id is null";
    get_lobbies(filter, pool).await
}

#[get("/get_joinable_lobbies/<player_id>")]
async fn get_joinable_lobbies(player_id: IdType, pool: &State<Pool>) -> Result<String, String> {
    let filter = &format!(
        "(player1_id is null or player2_id is null) and (player1_id != {p} or player2_id != {p})",
        p = player_id
    );
    get_lobbies(filter, pool).await
}

#[get("/get_joined_lobbies/<player_id>")]
async fn get_joined_lobbies(player_id: IdType, pool: &State<Pool>) -> Result<String, String> {
    let filter = &format!("player1_id = {p} or player2_id = {p}", p = player_id);
    get_lobbies(filter, pool).await
}

#[get("/gamelobby/<game_id>")]
async fn getgamelobby(game_id: IdType, pool: &State<Pool>) -> Result<String, String> {
    let query = format!(
        "SELECT player1_id, player2_id, game_name, game_started from gamelist WHERE game_id = {}",
        game_id
    );

    let mut conn = pool.inner().get_conn().expect("failed to connect to db");
    let games = conn
        .query_map(
            query,
            |(player1_id, player2_id, game_name, game_started)| GameLobby {
                game_id,
                player1_id,
                player2_id,
                game_name,
                game_started,
            },
        )
        .map_err(|_| "failed to get games from database")?;

    serde_json::to_string(&games[0]).map_err(|_| "serializing failed".to_owned())
}

// #[derive(FromForm)]
// struct CreateLobbyForm<'f> {
//     game_name: &'f str,
// }

// #[post("/create_game_lobby", data = "<form>")]
// fn create_game_lobby(form: Form<CreateLobbyForm<'_>>, cookies: &CookieJar<'_>, pool: &State<Pool>) {

#[post("/create_game_lobby", data = "<game_name>")]
fn create_game_lobby(game_name: String, cookies: &CookieJar<'_>, pool: &State<Pool>) -> String {
    // if cookies.get("session_id").is_none() {
    //     cookies.add(Cookie::build("session_id", rand::random::<IdType>().to_string()).finish())
    // }
    let session_id_string = match cookies.get("session_id") {
        Some(cookie) => cookie.value().to_owned(),
        None => {
            let random_id = rand::random::<IdType>().to_string();
            cookies.add(Cookie::build("session_id", random_id.clone()).finish());
            random_id
        }
    };
    let mut conn = pool.inner().get_conn().unwrap();
    let session_id: IdType = session_id_string.parse().unwrap();
    let new_game_lobby = GameLobby {
        game_id: rand::random::<IdType>(),
        player1_id: Some(session_id),
        player2_id: None,
        // game_name: form.game_name.to_owned(),
        game_name: game_name,
        game_started: false,
    };

    conn.exec_drop("INSERT INTO gamelist (game_id, player1_id, player2_id, game_name, game_started) VALUES (:game_id, :player1_id, :player2_id, :game_name, :game_started)",
     params! {"game_id" => new_game_lobby.game_id,
                "player1_id" => new_game_lobby.player1_id, 
                "player2_id" => new_game_lobby.player2_id,
                "game_name" => new_game_lobby.game_name,
            "game_started"=> new_game_lobby.game_started}).unwrap();

    println!(
        "Game_id comparison:\n{}\n{}",
        new_game_lobby.game_id,
        new_game_lobby.game_id.to_string()
    );
    new_game_lobby.game_id.to_string()
}

// #[derive(FromForm)]
// struct GameIdForm {
//     game_id: IdType,
// }

// #[post("/join", data = "<game_id_form>")]
// fn join(game_id_form: Form<GameIdForm>, pool: &State<Pool>, cookies: &CookieJar<'_>) -> String {
//     let mut conn = pool.inner().get_conn().unwrap();
//     match conn.exec_drop(
//         "UPDATE gamelist SET player2_id = :player2_id WHERE game_id=:game_id",
//         params! {"game_id" => game_id_form.game_id,
//         "player2_id" => cookies.get("session_id").unwrap().value()},
//     ) {
//         Ok(_) => "Succes!".to_owned(),
//         Err(_) => "Failed".to_owned(),
//     }
// }

#[get("/getid")]
fn getid(cookies: &CookieJar<'_>) -> String {
    match cookies.get("session_id") {
        Some(cookie) => cookie.value().to_owned(),
        None => {
            let random_id = rand::random::<IdType>().to_string();
            cookies.add(Cookie::build("session_id", random_id.clone()).finish());
            random_id
        }
    }
}

#[get("/join/<game_id>")]
fn join(game_id: IdType, pool: &State<Pool>, cookies: &CookieJar<'_>) -> String {
    let mut conn = pool.inner().get_conn().unwrap();
    match conn.exec_drop(
        "UPDATE gamelist SET player2_id = :player2_id WHERE game_id=:game_id",
        params! {"game_id" => game_id,
        "player2_id" => cookies.get("session_id").unwrap().value()},
    ) {
        Ok(_) => "Succes!".to_owned(),
        Err(_) => "Failed".to_owned(),
    }
}

#[post("/save_game", data = "<gamedata_json>")]
fn save_game(gamedata_json: Json<GameData>, pool: &State<Pool>) {
    let Json(gamedata) = gamedata_json;
    println!("Received dat data: \n {:?}", &gamedata);
    let mut conn = pool.inner().get_conn().unwrap();
    let turn_player_num: u8 = gamedata.turn_player.into();
    let win_status_num: Option<u8> = gamedata.win_status.map(Player::into);
    conn.exec_drop(
        "UPDATE games SET board = :board, turn_player = :turn_player, win_status = :win_status, winning_chips = :winning_chips WHERE game_id = :game_id",
        params! {"board" => serde_json::to_string(&gamedata.board).unwrap(),
    "turn_player" => turn_player_num,
    "win_status" => win_status_num,
    "winning_chips" => serde_json::to_string(&gamedata.winning_chips).unwrap(),
    "game_id" => gamedata.game_id})
    .unwrap();
    println!("Saved dat data");
}

#[post("/create_game", data = "<gamedata_json>")]
fn create_game(gamedata_json: Json<GameData>, pool: &State<Pool>) {
    println!("Received JSON: {:?}", gamedata_json);
    let Json(gamedata) = gamedata_json;
    let mut conn = pool.inner().get_conn().unwrap();
    let turn_player_num: u8 = gamedata.turn_player.into();
    let win_status_num: Option<u8> = gamedata.win_status.map(Player::into);
    conn.exec_drop(
        "INSERT INTO games (
            game_id, board, win_length, turn_player, win_status, winning_chips, player1_id, player2_id
        ) VALUES (:game_id, :board, :win_length, :turn_player, :win_status, :winning_chips, :player1_id, :player2_id)",
        params! {"game_id" => gamedata.game_id,
        "board" => serde_json::to_string(&gamedata.board).unwrap(),
        "win_length" => gamedata.win_length,
    "turn_player" => turn_player_num,
    "win_status" => win_status_num,
    "winning_chips" => serde_json::to_string(&gamedata.winning_chips).unwrap(),
    "player1_id" => gamedata.player1_id,
    "player2_id" => gamedata.player2_id
    })
    .unwrap();
}

// #[post("/create_game", data = "<gamedata_json>")]
// fn create_game(gamedata_json: String) {
//     println!("{}", gamedata_json);
// }

#[get("/gamedata/<game_id>")]
fn gamedata(game_id: IdType, pool: &State<Pool>) -> String {
    let mut conn = pool.inner().get_conn().unwrap();
    let results = conn
        .query_map(
            format!("SELECT * FROM games WHERE game_id = {}", game_id),
            |(
                game_id,
                board_json,
                win_length,
                turn_player_num,
                win_status_num,
                winning_chips_json,
                player1_id,
                player2_id,
            )| {
                let board_json: String = board_json; // TODO hacks to satisfy type checker. Is there a better way?
                let winning_chips_json: String = winning_chips_json;
                let turn_player_num: u8 = turn_player_num;
                let win_status_num: Option<u8> = win_status_num;

                GameData {
                    game_id,
                    board: serde_json::from_str(board_json.as_str()).unwrap(),
                    win_length,
                    turn_player: turn_player_num.try_into().unwrap(),
                    win_status: win_status_num.map(|num| num.try_into().unwrap()),
                    winning_chips: serde_json::from_str(winning_chips_json.as_str()).unwrap(),
                    player1_id,
                    player2_id,
                }
            }, //     GameData {
               //         game_id,
               //         board: serde_json::from_str(&board_json).unwrap(),
               //         win_length,
               //         turn_player: turn_player_num.try_into().unwrap(),
               //         win_status: winning_chips_json.map(|num| num.try_into().unwrap()),
               //         winning_chips: serde_json::from_str(&winning_chips_json).unwrap(),
               //         player1_id,
               //         player2_id,

               // }
        )
        .unwrap();

    serde_json::to_string(&results.get(0)).unwrap()
}

#[launch]
fn rocket() -> _ {
    let pwd = fs::read_to_string("pwd.txt").unwrap();
    let url = format!("mysql://root:{}@localhost:3306/connect_torus_db", pwd);
    let pool = Pool::new(url.as_str()).expect("failed to create pool");
    // let mut conn = pool
    //     .get_conn()
    //     .expect("failed to create connection to database");

    rocket::build()
        .mount("/", routes![index, getfile, redirect_ui])
        .mount(
            "/api",
            routes![
                getgamelist,
                create_game_lobby,
                join,
                getid,
                create_game,
                save_game,
                gamedata,
                getgamelobby,
                get_joinable_lobbies,
                get_joined_lobbies
            ],
        ) //
        .manage(pool)
        .register("/", catchers![not_found])
}
