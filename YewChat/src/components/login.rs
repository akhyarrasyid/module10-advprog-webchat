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
        <div class="min-h-screen bg-gradient-to-br from-blue-500 to-blue-700 flex items-center justify-center p-4">
            <div class="w-full max-w-md">
                <div class="bg-white rounded-2xl shadow-xl overflow-hidden">
                    <div class="bg-gradient-to-r from-blue-600 to-blue-500 p-8 text-center">
                        <h1 class="text-3xl font-bold text-white">{"Welcome to ChatApp"}</h1>
                        <p class="text-blue-100 mt-2">{"Connect with your friends in real-time"}</p>
                    </div>
                    
                    <div class="p-8">
                        <div class="mb-6">
                            <label class="block text-blue-800 text-sm font-bold mb-2" for="username">
                                {"Username"}
                            </label>
                            <input 
                                id="username"
                                oninput={oninput}
                                class="w-full px-4 py-3 rounded-lg border border-blue-200 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent transition-all duration-200"
                                placeholder="Enter your username"
                                autocomplete="off"
                            />
                        </div>
                        
                        <Link<Route> to={Route::Chat} classes="block w-full">
                            <button 
                                {onclick} 
                                disabled={username.len()<1}
                                class={classes!(
                                    "w-full",
                                    "py-3",
                                    "px-4",
                                    "rounded-lg",
                                    "font-bold",
                                    "text-white",
                                    "bg-gradient-to-r",
                                    "from-blue-500",
                                    "to-blue-600",
                                    "hover:from-blue-600",
                                    "hover:to-blue-700",
                                    "focus:outline-none",
                                    "focus:ring-2",
                                    "focus:ring-blue-400",
                                    "transform",
                                    "hover:scale-105",
                                    "transition-all",
                                    "duration-200",
                                    "shadow-md",
                                    "hover:shadow-lg",
                                    if username.len() < 1 {
                                        "opacity-50 cursor-not-allowed"
                                    } else {
                                        ""
                                    }
                                )}
                            >
                                {"Start Chatting"}
                                <svg class="w-5 h-5 inline-block ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7"></path>
                                </svg>
                            </button>
                        </Link<Route>>
                    </div>
                    
                    <div class="bg-blue-50 px-8 py-4 text-center">
                        <p class="text-blue-600 text-sm">
                            {"By joining, you agree to our "}
                            <a href="#" class="text-blue-700 hover:underline">{"Terms"}</a>
                            {" and "}
                            <a href="#" class="text-blue-700 hover:underline">{"Privacy Policy"}</a>
                        </p>
                    </div>
                </div>
                
                <div class="mt-6 text-center">
                    <p class="text-blue-100">
                        {"Need help? "}
                        <a href="#" class="text-white hover:underline">{"Contact support"}</a>
                    </p>
                </div>
            </div>
        </div>
    }
}