use std::eprint;
use std::io;
use std::io::BufRead;
use std::time::Duration;
use std::vec::Vec;

use std::cell::RefCell;

use anyhow::Result;
use futures::stream::StreamExt;
use reqwest::header;

thread_local!(
    static CL: RefCell<reqwest::Client> = RefCell::new({
        let mut headers = header::HeaderMap::new();
        headers.insert("Accept", header::HeaderValue::from_static("text/html"));
        reqwest::ClientBuilder::new()
            .use_rustls_tls()
            .default_headers(headers)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36")
            .timeout(Duration::new(10, 0))
            .build()
            .unwrap()
    });

    static PROGRESS: RefCell<usize> = RefCell::new(0);
);

fn print_progress(total: usize) {
    PROGRESS.with(|t| {
        let mut processed = t.borrow_mut();
        *processed += 1;
        eprint!("Processed: {}/{}\r", processed, total);
    });
}

async fn check(url: &str) -> Result<bool> {
    let res = CL.with(|client| client.borrow().get(url).send()).await?;
    // eprintln!("{} {}", res.status(), url);
    Ok(res.status() == 200)
}

#[tokio::main]
async fn main() {
    let urls = io::stdin()
        .lock()
        .lines()
        .filter_map(std::result::Result::ok)
        .collect::<Vec<String>>();
    let total = urls.len();

    let statuses = futures::stream::iter(urls.iter().map(|url| async move {
        let status = check(url).await;
        print_progress(total);

        match status {
            Ok(true) => true,
            _ => false,
        }
    }))
    .buffered(50)
    .collect::<Vec<bool>>()
    .await;

    urls.iter()
        .zip(statuses.iter())
        .filter(|(_url, &status)| !status)
        .for_each(|(url, _)| println!("{}", url));
}
