use reddit::{
    Client,
    RedditError,
    RedditResult,
};

// 25 is the default

async fn get_subreddit(name: &str) -> RedditResult<()> {
    let client = Client::new();
    let subreddit = client.get_subreddit(name, 100).await?;
    println!("{}", subreddit.data.children.len());
    Ok(())
}

#[tokio::test]
async fn subreddit_dankmemes() {
    get_subreddit("dankmemes").await.unwrap();
}

#[tokio::test]
async fn fake_subreddit() {
    let client = Client::new();
    let err = client.get_subreddit("gfdghfj", 25).await.unwrap_err();
    assert!(err.is_not_found(), "err = {:#?}", err);
}

#[tokio::test]
async fn subreddit_forbiddensnacks() {
    get_subreddit("forbiddensnacks").await.unwrap();
}

#[tokio::test]
async fn subreddit_cursedimages() {
    get_subreddit("cursedimages").await.expect("Valid");
}

#[tokio::test]
async fn subreddit_meow_irl() {
    get_subreddit("MEOW_IRL").await.expect("Valid");
}

#[tokio::test]
async fn subreddit_cuddleroll() {
    match get_subreddit("cuddleroll").await {
        Ok(_) => (),
        Err(RedditError::Json(e, _)) => panic!("{:#?}", e),
        Err(e) => panic!("{:#?}", e),
    }
}

#[tokio::test]
async fn subreddit_cromch() {
    get_subreddit("cromch").await.expect("Valid");
}

#[tokio::test]
async fn subreddit_cats() {
    get_subreddit("cats").await.expect("Valid");
}

#[tokio::test]
async fn subreddit_cursed_images() {
    get_subreddit("cursed_images").await.expect("Valid");
}

#[tokio::test]
async fn subreddit() {
    match get_subreddit("aww").await {
        Ok(_) => (),
        Err(RedditError::Json(e, _)) => panic!("{:#?}", e),
        Err(e) => panic!("{:#?}", e),
    }
}
