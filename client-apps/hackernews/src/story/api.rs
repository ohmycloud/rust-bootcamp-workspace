use anyhow::Result;
use crate::story::item::StoryItem;
use futures::future::join_all;
use crate::story::comment::Comment;
use crate::story::page::StoryData;

pub async fn get_story_item_by_id(id: i64) -> Result<StoryItem> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    let item: StoryItem = reqwest::get(&url).await?.json().await?;

    Ok(item)
}

// only retrieve top comments
pub async fn get_top_stories(number: usize) -> Result<Vec<StoryItem>> {
    let url = "https://hacker-news.firebaseio.com/v0/topstories.json";
    let ids: Vec<i64> = reqwest::get(url).await?.json().await?;
    let story_futures = ids
        .into_iter()
        .take(number)
        .map(|id| get_story_item_by_id(id));

    let stories = join_all(story_futures)
        .await
        .into_iter()
        .filter_map(|item| item.ok())
        .collect::<Vec<StoryItem>>();
    Ok(stories)
}

pub async fn get_comment_by_id(id: i64) -> Result<Comment> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
    let comment: Comment = reqwest::get(&url).await?.json().await?;

    Ok(comment)
}

pub async fn get_story_comment(item: StoryItem) -> Result<StoryData> {
    let comment_futures = item
        .kids
        .iter()
        .map(|id| get_comment_by_id(*id));

    let comments = join_all(comment_futures)
        .await
        .into_iter()
        .filter_map(|comment| comment.ok())
        .collect::<Vec<Comment>>();

    Ok(StoryData {
        item,
        comments
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_top_stories_should_work() -> Result<()> {
        let stories = get_top_stories(3).await?;
        assert_eq!(stories.len(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn get_comment_by_id_should_work() -> Result<()> {
        let id = get_top_stories(1)
            .await?[0]
            .kids[0];

        let comment = get_comment_by_id(id).await?;
        assert_eq!(comment.id, id);
        Ok(())
    }
}