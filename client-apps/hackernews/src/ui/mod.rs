#![allow(non_snake_case)]

mod story_item;
mod story_comment;
mod stories;

use dioxus::prelude::*;
use crate::story::StoryData;
use crate::ui::stories::Stories;
use crate::ui::story_comment::StoryComment;

pub fn App() -> Element {
    use_context_provider(|| Signal::new(CommentState::Unset));
    rsx! {
        main { class: "flex w-full h-full shadow-lg rounded-3xl",
            section { class: "flex flex-col w-4/12 h-full pt-3 overflow-y-scroll bg-gray-50",
               Stories {}
            }
            section { class: "flex flex-col w-8/12 px-4 bg-white rounded-r-3xl",
                section {
                   Comments {}
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CommentState {
    Unset,
    Loading,
    Loaded(StoryData),
}

#[component]
pub fn Comments() -> Element {
    let comment_state = use_context::<Signal<CommentState>>();

    match comment_state() {
        CommentState::Unset => rsx! {},
        CommentState::Loading => rsx! {
            div {
                class: "mt-6",
                p { "Loading comments..." }
            }
        },
        CommentState::Loaded(data) => rsx! {
            ul {
                for comment in data.comments {
                    StoryComment { comment }
                }
            }
        },
    }
}