use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).clone())
    };

    html! {
        <div style="text-align: center; margin-top: 10vh; border: 2px solid #005500; padding: 50px; display: inline-block; box-shadow: 0 0 20px rgba(57,255,20,0.1);">
            <h1 style="text-shadow: 0 0 10px #39ff14; letter-spacing: 5px; margin-bottom: 5px;">{"[ SYS.NET_TERMINAL ]"}</h1>
            <p style="color: #00aa00; margin-bottom: 30px;">{"> AWAITING USER IDENTIFICATION..."}</p>

            <input type="text" placeholder="root@alias:~$" value={(*username).clone()} oninput={oninput} style="margin-right: 10px;" />

            <Link<Route> to={Route::Chat}>
                <button disabled={username.len() < 2} onclick={onclick}>{"INITIATE"}</button>
            </Link<Route>>
        </div>
    }
}
