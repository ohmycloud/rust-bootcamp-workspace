use dioxus::prelude::*;
use crate::story::get_top_stories;
use crate::ui::story_item::StoryItemElement;

#[component]
pub fn Stories() -> Element {
    let stories = use_resource(move || get_top_stories(20));

    match &*stories.read_unchecked() {
        Some(Ok(stories)) => rsx! {
            ul {
                for story in stories {
                    StoryItemElement {
                        story: story.clone()
                    }
                }
            }
        },
        Some(Err(err)) => rsx! {
            div {
                class: "mt-6 text-red-500",
                p { "Failed to fetch stories" }
                p { "{err}" }
            }
        },
        None => rsx! {
            div {
                class: "mt-6",
                p { "Loading stories" }
            }
        }
    }
}