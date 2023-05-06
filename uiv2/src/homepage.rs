use reqwasm::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::use_navigator;

use crate::Pages;

#[function_component]
pub fn HomePage() -> Html {
    let input_value_handle = use_state(String::default);
    let input_value = (*input_value_handle).clone();

    let on_change = {
        let input_value_handle = input_value_handle.clone();

        Callback::from(move |e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                input_value_handle.set(input.value());
            }
        })
    };

    let navigator = use_navigator().unwrap();
    let input_value_clone = input_value.clone();
    let on_submit = Callback::from(move |_| {
        let input_value_clone = input_value_clone.clone();
        let navigator = navigator.clone();
        log::info!("{}", input_value_clone);
        spawn_local(async move {
            let game_id = Request::post("/api/create_game_lobby")
                .body(input_value_clone)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

            log::info!("Game_id on front end: {}", &game_id);
            // let game_id: u64 = game_id.parse().unwrap();
            navigator.push(&Pages::Lobby { game_id })
        });
    });

    let navigator = use_navigator().unwrap();
    let to_local_game = Callback::from(move |_| navigator.push(&Pages::Local));
    html! {
        <div class="mainpage">
        <a href="/gamelist">
            <button class="smallblock" style="cursor:pointer">{"Find game"}</button>
        </a>

        // <form action="/api/create_game_lobby" method="post">
        //     <label for="game_name">{"Enter game name"}</label>
        //     <input type="text" id="game_name" name="game_name"/>
        //     <input type="submit" value="Submit"/>
        // </form>
        <p>{"Enter game name"}</p>
        <input onchange={on_change}
        id="cautious-input"
        type="text"
        value={input_value.clone()}
        />
        <button class="smallblock" style="cursor:pointer" onclick={on_submit}> {"Create game"} </button>


        // <div oninput={oninput}>
        // { "hi" }
        // <input type="text" />
        // </div>
        <button onclick={to_local_game} class="smallblock" style="cursor:pointer">{"Play local game"}</button>
        </div>
    }
}
