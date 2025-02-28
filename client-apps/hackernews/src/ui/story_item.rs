#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use crate::story::{get_story_comment, StoryData, StoryItem};
use crate::ui::CommentState;

#[component]
pub fn StoryItemElement(story: StoryItem) -> Element {
    let comments_state = use_context::<Signal<CommentState>>();
    // cache of the already loaded comments: Option<StoryData>
    let full_story = use_signal(|| None);

    rsx! {
        li { class: "px-3 py-5 transition border-b hover:bg-indigo-100",
            a { href: "#", class: "flex items-center justify-between",
                h3 { class: "text-lg font-semibold", "{story.title}" }
                p { class: "text-gray-400 text-md" }
            }
            div { class: "italic text-gray-400 text-md",
                span { "{story.score} points by {story.by} {story.time} | " }
                a { href: "#",
                    onclick: move |event| {
                        event.prevent_default();
                        info!("Clicked on story: {} with event: {:?}", story.title, event);
                        load_comments(comments_state, full_story, story.clone())
                    },
                    "{story.kids.len()} comments" }
            }
       }
    }
}

async fn load_comments(
    mut comment_state: Signal<CommentState>,
    mut full_story: Signal<Option<StoryData>>,
    story: StoryItem,
) {
    // if the comments are already loaded, just change comments_state and return
    if let Some(story_data) = full_story.as_ref() {
        *comment_state.write() = CommentState::Loaded(story_data.clone());
        return;
    }

    // if no set comments_state to Loading and fetch the comments
    *comment_state.write() = CommentState::Loading;

    if let Ok(story_data) = get_story_comment(story).await {
        *comment_state.write() = CommentState::Loaded(story_data.clone());
        *full_story.write() = Some(story_data);
    }
}