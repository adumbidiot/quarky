use hyper::rt::{
    self,
    Future,
};
use reddit::{
    Client,
    PostHint,
    RedditError,
    RedditResult,
};
use tokio::runtime::Runtime;

// 25 is the default

fn get_subreddit(name: &str) -> RedditResult<()> {
    let mut rt = Runtime::new().expect("Runtime init");
    let client = Client::new();
    let fut = client
        .get_subreddit(name, 100)
        .map(|res| println!("{}", res.data.children.len()));
    drop(client);
    rt.block_on(fut)
}

#[test]
fn subreddit_dankmemes() {
    get_subreddit("dankmemes").unwrap();
}

#[test]
fn fake_subreddit() {
    let client = Client::new();
    let fut = client
        .get_subreddit("gfdghfj", 25)
        .map(|_res| panic!("Should fail"))
        .map_err(|e| assert!(e.is_not_found()));
    drop(client);
    rt::run(fut);
}

#[test]
fn subreddit_forbiddensnacks() {
    get_subreddit("forbiddensnacks").unwrap();
}

#[test]
fn subreddit_cursedimages() {
    get_subreddit("cursedimages").expect("Valid");
}

#[test]
fn subreddit_meow_irl() {
    get_subreddit("MEOW_IRL").expect("Valid");
}

#[test]
fn subreddit_cuddleroll() {
    match get_subreddit("cuddleroll") {
        Ok(_) => (),
        Err(RedditError::Json(e, _)) => panic!("{:#?}", e),
        Err(e) => panic!("{:#?}", e),
    }
}

#[test]
fn subreddit_cromch() {
    get_subreddit("cromch").expect("Valid");
}

#[test]
fn subreddit_cats() {
    get_subreddit("cats").expect("Valid");
}

#[test]
fn subreddit_cursed_images() {
    get_subreddit("cursed_images").expect("Valid");
}

#[test]
fn subreddit() {
    match get_subreddit("aww") {
        Ok(_) => (),
        Err(RedditError::Json(e, _)) => panic!("{:#?}", e),
        Err(e) => panic!("{:#?}", e),
    }
}
