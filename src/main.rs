use std::eprint;
use std::io;
use std::io::BufRead;
use std::time::Duration;

use std::cell::RefCell;

use anyhow::Result;
use clap::Clap;
use futures::future;
use futures::stream::StreamExt;
use reqwest::{header, Client};

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "Imran Khan")]
struct Config {
    #[clap(
        short = "u",
        long = "user-agent",
        default_value = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.141 Safari/537.36"
    )]
    user_agent: String,
    #[clap(
        short = "t",
        long = "timeout",
        default_value = "10",
        parse(try_from_str)
    )]
    timeout: u64,
}

thread_local!(
    static PROGRESS: RefCell<usize> = RefCell::new(0);
);

fn print_progress(total: usize) {
    PROGRESS.with(|t| {
        let mut processed = t.borrow_mut();
        *processed += 1;
        eprint!("Processed: {}/{}\r", processed, total);
    });
}

async fn check(client: &Client, url: &str) -> Result<bool> {
    let res = client.get(url).send().await?;
    // eprintln!("{} {}", res.status(), url);
    Ok(res.status() == 200)
}

#[tokio::main]
async fn main() {
    let config = Config::parse();

    let urls = io::stdin()
        .lock()
        .lines()
        .flatten()
        .collect::<Vec<String>>();
    let total = urls.len();

    let client = {
        let mut headers = header::HeaderMap::new();
        headers.insert("Accept", header::HeaderValue::from_static("text/html"));
        reqwest::ClientBuilder::new()
            .use_rustls_tls()
            .default_headers(headers)
            .user_agent(config.user_agent)
            .timeout(Duration::from_secs(config.timeout))
            .build()
            .unwrap()
    };

    let cl = &client;
    futures::stream::iter(urls.iter().map(|url| async move {
        let status = check(cl, url).await;
        print_progress(total);

        match status {
            Ok(true) => (true, url),
            _ => (false, url),
        }
    }))
    .buffered(50)
    .filter(|(status, _url)| future::ready(!status))
    .for_each(|(_, url)| {
        println!("{}", url);
        future::ready(())
    })
    .await;

    eprintln!("");
}
