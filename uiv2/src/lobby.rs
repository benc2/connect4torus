use crate::{connectgame::GameData, gamelist::GameLobby, Pages};
use reqwasm::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(PartialEq, Properties)]
pub struct GameLobbyProps {
    pub game_id: u64,
}

// #[function_component]
// pub fn GameLobbyView(game_lobby_props: &GameLobbyProps) -> Html {
//     let game_id = game_lobby_props.game_id;
//     let gamelobby_state = use_state(|| {
//         Some(GameLobby {
//             game_id: 42,
//             player1_id: Some(420),
//             player2_id: Some(69),
//             game_name: "please just work".to_owned(),
//         })
//     });
//     let gamelobby_state_clone = gamelobby_state.clone();
//     let gamelobby_state_clone2 = gamelobby_state.clone();

//     log::info!("WHAT?!? {:?}", &*gamelobby_state);

//     spawn_local(async move {
//         let gamelobby: GameLobby = Request::get(&format!("/api/gamelobby/{}", game_id))
//             .send()
//             .await
//             .unwrap()
//             .json()
//             .await
//             .unwrap();
//         log::info!("Got lobby: {:?}", &gamelobby);
//         gamelobby_state.set(Some(gamelobby));
//         // gamelobby_state.set(Some(GameLobby {
//         //     game_id: 42,
//         //     player1_id: Some(420),
//         //     player2_id: Some(69),
//         //     game_name: "please just work".to_owned(),
//         // }));
//         log::info!("Inside future: {:?}", *gamelobby_state);
//     });

//     html! {"Cheese"}

//     // log::info!("Outside future cloned: {:?}", *gamelobby_state_clone);
//     // let gamelobby = gamelobby_state_clone.as_ref().unwrap().clone();
//     // let startable = gamelobby.number_players_joined() == 2;
//     // let navigator = use_navigator().unwrap();
//     // let start_game = Callback::from(move |_| {
//     //     if !startable {
//     //         return;
//     //     }
//     //     let gamedata = GameData::new(
//     //         7,
//     //         6,
//     //         4,
//     //         gamelobby.game_id,
//     //         gamelobby.player1_id.unwrap(), // is safe due to startable check
//     //         gamelobby.player2_id.unwrap(),
//     //     );
//     //     spawn_local(async move {
//     //         Request::post("/api/create_game")
//     //             .body(serde_json::to_string(&gamedata).unwrap())
//     //             .send()
//     //             .await
//     //             .unwrap();
//     //     });
//     //     navigator.push(&Pages::Game {
//     //         game_id: gamelobby.game_id,
//     //     });
//     // });
//     // html! {
//     //     <>
//     //     <p>{format!("{} players have joined", gamelobby.number_players_joined())}</p>
//     //     <button class={match startable {
//     //         true => "greenbutton",
//     //         false => "graybutton"
//     //     }} onclick = {start_game}> {"Start game"} </button>
//     //     </>
//     // }
// }

#[derive(PartialEq)]
enum FetchState {
    NotFetching,
    Fetching,
    Success,
    Failure,
}

#[function_component]
pub fn GameLobbyView(game_lobby_props: &GameLobbyProps) -> Html {
    let game_id = game_lobby_props.game_id;
    let gamelobby_state = use_state(|| None);
    let fetch_state = use_state(|| FetchState::NotFetching);
    let gamelobby_state_clone = gamelobby_state.clone();
    if *fetch_state == FetchState::NotFetching {
        fetch_state.set(FetchState::Fetching);
        spawn_local(async move {
            // let gamelobby: GameLobby = Request::get(&format!("/api/gamelobby/{}", game_id))
            //     .send()
            //     .await
            //     .unwrap()
            //     .json()
            //     .await
            //     .unwrap();
            // gamelobby_state.set(Some(gamelobby));

            log::info!(
                "JSON analysis: {}",
                Request::get(&format!("/api/gamelobby/{}", game_id))
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap()
            );

            let gamelobby_result: Result<GameLobby, _> =
                Request::get(&format!("/api/gamelobby/{}", game_id))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await;

            match gamelobby_result {
                Ok(gamelobby) => gamelobby_state.set(Some(gamelobby)),
                Err(err) => log::info!("Got error: {}", err),
            }
            // let log::info!("Yeet!");
        });
        fetch_state.set(FetchState::Success);
    }
    let navigator = use_navigator().unwrap();
    let body_html = match gamelobby_state_clone.as_ref() {
        Some(gamelobby) => {
            let gamelobby = gamelobby_state_clone.as_ref().unwrap().clone();

            let startable = gamelobby.number_players_joined() == 2;

            let start_game = Callback::from(move |_| {
                if !startable {
                    return;
                }

                if gamelobby.game_started {
                    navigator.push(&Pages::Game { game_id });
                    return;
                }

                let gamedata = GameData::new(
                    7,
                    6,
                    4,
                    gamelobby.game_id,
                    gamelobby.player1_id.unwrap(), // is safe due to startable check
                    gamelobby.player2_id.unwrap(),
                );
                spawn_local(async move {
                    Request::post("/api/create_game")
                        .body(serde_json::to_string(&gamedata).unwrap())
                        .send()
                        .await
                        .unwrap();
                });
                navigator.push(&Pages::Game {
                    game_id: gamelobby.game_id,
                });
            });
            html! {
                <>
                <p>{format!("{} players have joined", gamelobby.number_players_joined())}</p>
                <button class={match startable {
                    true => "greenbutton",
                    false => "graybutton"
                }} onclick = {start_game}> {"Start game"} </button>
                </>
            }
        }
        None => html! {"Please wait..."},
    };

    html! {<div class="smallblock">
    {body_html}
    </div>}
}
