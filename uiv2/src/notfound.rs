use yew::prelude::*;

#[function_component]
pub fn NotFoundPage() -> Html {
    html! {
        <>
        <h2>{"404: Page not found"}</h2>
        <p>{"It seems you are looking in the wrong place!"}</p>
        </>
    }
}
