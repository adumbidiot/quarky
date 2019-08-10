use hyper::rt::{
    self,
    Future,
};
use reddit::{
    Client,
    PostHint,
    RedditError,
};

//25 is the default

#[test]
fn subreddit_dankmemes() {
    let client = Client::new();
    let fut = client
        .get_subreddit("dankmemes", 25)
        .map(|res| {
            println!(
                "{}",
                res.data
                    .children
                    .iter()
                    .filter(|child| child.data.post_hint == PostHint::Image)
                    .collect::<Vec<_>>()
                    .len()
            )
        })
        .map_err(|e| panic!("{:#?}", e));
    drop(client);
    rt::run(fut);
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
fn subreddit() {
    let client = Client::new();
    let fut = client
        .get_subreddit("forbiddensnacks", 25)
        .map(|res| {
            println!(
                "{}",
                res.data
                    .children
                    .iter()
                    .filter(|child| child.data.post_hint == PostHint::Image)
                    .collect::<Vec<_>>()
                    .len()
            )
        })
        .map_err(|e| panic!("{:#?}", e));
    drop(client);
    rt::run(fut);
}
