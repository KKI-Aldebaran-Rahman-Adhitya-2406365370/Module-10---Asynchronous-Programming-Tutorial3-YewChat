use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div style="display: flex; width: 100vw; height: 100vh; background-color: #050a05; color: #39ff14; font-family: 'Courier New', Courier, monospace; overflow: hidden;">

                // COLUMN A: Users List
                <div style="flex: none; width: 280px; background-color: #000000; border-right: 2px solid #005500; display: flex; flex-direction: column;">
                    <div style="font-size: 1.25rem; padding: 15px; border-bottom: 2px solid #005500; text-shadow: 0 0 10px #39ff14; letter-spacing: 2px; font-weight: bold;">
                        {"> CONNECTED_NODES"}
                    </div>
                    <div style="overflow-y: auto; flex-grow: 1;">
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div style="display: flex; align-items: center; margin: 15px; background-color: #001100; border: 1px solid #004400; padding: 10px; box-shadow: 0 0 8px rgba(57,255,20,0.15);">
                                        <div>
                                            // The filter property turns the standard avatars into green glowing holograms
                                            <img style="width: 45px; height: 45px; border-radius: 50%; border: 1px solid #39ff14; filter: sepia(100%) hue-rotate(70deg) saturate(300%);" src={u.avatar.clone()} alt="avatar"/>
                                        </div>
                                        <div style="flex-grow: 1; padding-left: 15px;">
                                            <div style="font-size: 0.95rem; font-weight: bold; text-transform: uppercase;">
                                                {u.name.clone()}
                                            </div>
                                            <div style="font-size: 0.75rem; color: #00aa00; margin-top: 3px;">
                                                {"[SECURE_LINK_ACTIVE]"}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                // COLUMN B: Chat Area
                <div style="flex-grow: 1; display: flex; flex-direction: column; height: 100vh;">

                    // Header
                    <div style="width: 100%; height: 60px; border-bottom: 2px solid #005500; display: flex; align-items: center; padding: 0 20px; background-color: #020502;">
                        <div style="font-size: 1.25rem; text-shadow: 0 0 10px #39ff14; font-weight: bold;">{"[ SYS.NET_RELAY_ESTABLISHED ]"}</div>
                    </div>

                    // Messages Scroll Area
                    <div style="width: 100%; flex-grow: 1; overflow-y: auto; padding: 25px; background-color: #050a05; display: flex; flex-direction: column;">
                        {
                            self.messages.iter().map(|m| {
                                // Safely lookup user to prevent panic if a user disconnects but their message remains
                                let user_avatar = self.users.iter().find(|u| u.name == m.from).map(|u| u.avatar.clone()).unwrap_or_else(|| "https://avatars.dicebear.com/api/adventurer-neutral/unknown.svg".to_string());

                                html!{
                                    <div style="display: flex; align-items: flex-start; width: 65%; background-color: #001100; margin-bottom: 20px; padding: 15px; border: 1px solid #005500; border-left: 4px solid #39ff14; box-shadow: 0 2px 10px rgba(57, 255, 20, 0.05);">
                                        <img style="width: 40px; height: 40px; border-radius: 50%; filter: sepia(100%) hue-rotate(70deg) saturate(300%); margin-right: 15px; border: 1px solid #005500;" src={user_avatar} alt="avatar"/>
                                        <div style="width: 100%;">
                                            <div style="font-size: 0.9rem; font-weight: bold; color: #00ff00; margin-bottom: 8px; border-bottom: 1px dashed #004400; padding-bottom: 4px;">
                                                {"> "}{m.from.clone()}
                                            </div>
                                            <div style="font-size: 1rem; color: #39ff14; line-height: 1.4;">
                                                // Preserving your exact GIF rendering logic
                                                if m.message.ends_with(".gif") {
                                                    <img style="margin-top: 10px; border: 1px solid #005500; max-width: 100%;" src={m.message.clone()}/>
                                                } else {
                                                    <span>{m.message.clone()}</span>
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    // Input Area
                    <div style="width: 100%; height: 80px; display: flex; align-items: center; padding: 0 25px; background-color: #000000; border-top: 2px solid #005500;">
                        <span style="font-weight: bold; margin-right: 15px; font-size: 1.2rem;">{">"}</span>

                        // Bound perfectly to your self.chat_input NodeRef
                        <input ref={self.chat_input.clone()} type="text" placeholder="Transmit payload..." style="flex-grow: 1; padding: 15px; background-color: transparent; border: 1px solid #008800; color: #39ff14; outline: none; font-family: 'Courier New', Courier, monospace; font-size: 1rem; box-shadow: inset 0 0 10px rgba(0,255,0,0.1);" name="message" required=true />

                        // Bound perfectly to your submit callback
                        <button onclick={submit} style="padding: 15px 30px; margin-left: 20px; background-color: #003300; border: 1px solid #39ff14; color: #39ff14; cursor: pointer; font-weight: bold; font-size: 1rem; font-family: 'Courier New', Courier, monospace; box-shadow: 0 0 15px rgba(57,255,20,0.3); text-transform: uppercase;">
                            {"EXEC"}
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
