#![allow(non_snake_case)]

use dioxus::dioxus_core::Element;
use dioxus::prelude::*;
use crate::story::Comment;

#[component]
pub fn StoryComment(comment: Comment) -> Element {
    rsx! {
        li {
            article { class: "p-4 leading-7 tracking-wider text-gray-500",
            span { "{comment.by} {comment.time} | next [-]" }
            div { dangerous_inner_html: comment.text }
            }
        }
    }
}