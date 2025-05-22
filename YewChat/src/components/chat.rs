use serde::{Deserialize, Serialize};
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    SetTyping(bool),
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
    timestamp: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
    Typing,
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
    is_typing: bool,
    last_seen: Option<i64>,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    is_typing: bool,
    timeout_handle: Option<Rc<dyn Fn()>>,
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
            is_typing: false,
            timeout_handle: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                                    "https://api.dicebear.com/7.x/identicon/svg?seed={}&backgroundType=gradientLinear&backgroundColor=b6e3f4,c0aede,d1d4f9",
                                    u
                                ),
                                is_typing: false,
                                last_seen: None,
                            })
                            .collect();
                        true
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        true
                    }
                    MsgTypes::Typing => {
                        if let Some(username) = msg.data {
                            if let Some(user) = self.users.iter_mut().find(|u| u.name == username) {
                                user.is_typing = true;
                                let link = ctx.link().clone();
                                let handle = Rc::new(move || {
                                    link.send_message(Msg::SetTyping(false));
                                });
                                self.timeout_handle = Some(handle.clone());
                                gloo::timers::callback::Timeout::new(3000, move || {
                                    handle();
                                }).forget();
                                return true;
                            }
                        }
                        false
                    }
                    _ => false,
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = input.value();
                    if !message.is_empty() {
                        let message_data = WebSocketMessage {
                            message_type: MsgTypes::Message,
                            data: Some(message.clone()),
                            data_array: None,
                        };
                        if let Err(e) = self
                            .wss
                            .tx
                            .clone()
                            .try_send(serde_json::to_string(&message_data).unwrap())
                        {
                            log::debug!("error sending to channel: {:?}", e);
                        }
                        input.set_value("");
                        self.is_typing = false;
                    }
                }
                false
            }
            Msg::SetTyping(typing) => {
                self.is_typing = typing;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let oninput = ctx.link().batch_callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut msgs = vec![];
            
            if !input.value().is_empty() {
                msgs.push(Msg::SetTyping(true));
            } else {
                msgs.push(Msg::SetTyping(false));
            }
            
            msgs
        });

        html! {
            <div class="flex h-screen w-screen bg-gradient-to-br from-blue-50 to-blue-100 overflow-hidden">
                <div class="flex-none w-72 h-full bg-white shadow-lg transform transition-all duration-300 hover:shadow-xl">
                    <div class="p-4 border-b border-blue-100">
                        <div class="text-2xl font-bold text-blue-800">{"Active Users"}</div>
                        <div class="text-sm text-blue-400">{"Online: "}{self.users.len()}</div>
                    </div>
                    <div class="overflow-y-auto h-[calc(100%-60px)]">
                        {for self.users.iter().map(|u| {
                            html!{
                                <div class="flex items-center p-3 hover:bg-blue-50 transition-colors duration-200">
                                    <div class="relative">
                                        <img class="w-12 h-12 rounded-full border-2 border-blue-200" src={u.avatar.clone()} alt="avatar"/>
                                        <div class="absolute bottom-0 right-0 w-3 h-3 bg-green-400 rounded-full border-2 border-white"></div>
                                    </div>
                                    <div class="ml-3">
                                        <div class="font-medium text-blue-800">{u.name.clone()}</div>
                                        <div class="text-xs text-blue-400">
                                            {if u.is_typing {
                                                html!{<span class="text-blue-500">{"typing..."}</span>}
                                            } else {
                                                html!{"online"}
                                            }}
                                        </div>
                                    </div>
                                </div>
                            }
                        })}
                    </div>
                </div>

                <div class="flex-1 flex flex-col h-full">
                    <div class="p-4 border-b border-blue-100 bg-white shadow-sm">
                        <div class="text-xl font-bold text-blue-800 flex items-center">
                            <svg class="w-6 h-6 mr-2 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"></path>
                            </svg>
                            {"Group Chat"}
                        </div>
                    </div>

                    <div class="flex-1 overflow-y-auto p-4 bg-gradient-to-b from-blue-50 to-blue-100">
                        <div class="space-y-4">
                            {for self.messages.iter().map(|m| {
                                let default_profile = UserProfile {
                                    name: m.from.clone(),
                                    avatar: format!(
                                        "https://api.dicebear.com/7.x/identicon/svg?seed={}&backgroundType=gradientLinear&backgroundColor=b6e3f4,c0aede,d1d4f9",
                                        m.from
                                    ),
                                    is_typing: false,
                                    last_seen: None,
                                };
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap_or(&default_profile);
                                
                                html!{
                                    <div class="flex items-start">
                                        <img class="w-10 h-10 rounded-full border-2 border-blue-200 mt-1" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="ml-3 max-w-xs lg:max-w-md">
                                            <div class="text-sm font-semibold text-blue-700">{m.from.clone()}</div>
                                            <div class="mt-1 px-4 py-2 bg-white rounded-lg shadow-sm text-sm text-gray-700">
                                                {if m.message.ends_with(".gif") {
                                                    html!{<img class="mt-1 rounded-lg" src={m.message.clone()}/>}
                                                } else {
                                                    html!{m.message.clone()}
                                                }}
                                            </div>
                                            <div class="text-xs text-blue-400 mt-1">
                                                {"Just now"}
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    </div>

                    <div class="p-4 border-t border-blue-100 bg-white">
                        <div class="flex items-center">
                            <input 
                                ref={self.chat_input.clone()} 
                                type="text" 
                                placeholder="Type your message..." 
                                class="flex-1 py-3 px-4 rounded-full bg-blue-50 focus:outline-none focus:ring-2 focus:ring-blue-300 focus:bg-white transition-all duration-200"
                                oninput={oninput}
                                onkeypress={ctx.link().batch_callback(|e: KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        vec![Msg::SubmitMessage]
                                    } else {
                                        vec![]
                                    }
                                })}
                            />
                            <button 
                                onclick={submit} 
                                class={classes!(
                                    "ml-3",
                                    "p-3",
                                    "bg-gradient-to-r",
                                    "from-blue-500",
                                    "to-blue-600",
                                    "rounded-full",
                                    "shadow-md",
                                    "hover:shadow-lg",
                                    "transform",
                                    "hover:scale-105",
                                    "transition-all",
                                    "duration-200",
                                    "focus:outline-none"
                                )}
                            >
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
                                </svg>
                            </button>
                        </div>
                        {if self.is_typing {
                            html!{
                                <div class="text-xs text-blue-500 mt-2 ml-2">
                                    {"You're typing..."}
                                </div>
                            }
                        } else {
                            html!{}
                        }}
                    </div>
                </div>
            </div>
        }
    }
}