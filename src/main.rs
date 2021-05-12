mod filters;
mod item;
use crate::filters::DateRangeFilter;
use crate::item::Item;
use anyhow::{Context, Result};
use futures::{future, StreamExt};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hncat",
    about = "Grab Hacker News items from the API in parallel."
)]
struct Opt {
    #[structopt(flatten)]
    date_filter: filters::DateRangeFilter,

    #[structopt(flatten)]
    since: filters::RelativeRangeFilter,

    #[structopt(flatten)]
    id_filter: filters::IDRangeFilter,

    /// Limit the number of outputted rows to this
    #[structopt(long)]
    limit: Option<usize>,

    /// Make this many concurrent requests
    #[structopt(long, default_value = "200")]
    concurrency: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let options: Opt = Opt::from_args();

    // This sucks: https://github.com/paritytech/fdlimit/pull/9
    fdlimit::raise_fd_limit();

    let client = reqwest::Client::new();
    let max_id = get_json(
        &client,
        "https://hacker-news.firebaseio.com/v0/maxitem.json".into(),
    )
    .await?;

    // Create a range
    let range = match options.id_filter {
        filters::IDRangeFilter {
            start_id: Some(start),
            stop_id: Some(stop),
        } => start..stop,
        filters::IDRangeFilter {
            start_id: Some(start),
            stop_id: None,
        } => start..max_id,
        filters::IDRangeFilter {
            start_id: None,
            stop_id: Some(stop),
        } => 0..stop,
        filters::IDRangeFilter {
            start_id: None,
            stop_id: None,
        } => 0..max_id,
    }
    .rev();

    let take_until_date = match (&options.date_filter, &options.since) {
        (
            filters::DateRangeFilter {
                start_date: None, ..
            },
            filters::RelativeRangeFilter { since: None, .. },
        ) => None,
        (
            filters::DateRangeFilter { start_date, .. },
            filters::RelativeRangeFilter { since: None, .. },
        ) => start_date.as_ref(),
        (
            filters::DateRangeFilter {
                start_date: None, ..
            },
            filters::RelativeRangeFilter { since, .. },
        ) => since.as_ref(),
        _ => unreachable!(),
    };

    let len = range.len();
    let total_bar = ProgressBar::new(len as u64);
    total_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {percent}% [{wide_bar:.cyan/blue}] {per_sec} p/s {pos}/{len} (ETA: {eta_precise})")
            .progress_chars("#>-"),
    );

    let mut errors: i32 = 0;

    let mut stream = futures::stream::iter(range.progress_with(total_bar))
        .map(|id: u32| {
            let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);
            get_json::<item::Item>(&client, url)
        })
        .buffered(options.concurrency)
        .filter_map(|item| {
            future::ready(match item {
                Ok(item) => Some(item),
                Err(_) => {
                    errors += 1;
                    None
                }
            })
        })
        // We iterate through IDs backwards. So we skip while item.time > end_date, and
        // take while item.time > start_date
        .skip_while(|item| {
            future::ready(match options.date_filter {
                DateRangeFilter { end_date: None, .. } => false,
                DateRangeFilter {
                    end_date: Some(end_date),
                    ..
                } => match item {
                    Item::LiveItem { time, .. } => time > &end_date,
                    Item::DeletedItem { time, .. } => time > &end_date,
                },
            })
        })
        .take_while(|item| {
            future::ready(match take_until_date {
                None => true,
                Some(start_date) => match item {
                    Item::LiveItem { time, .. } => time >= start_date,
                    Item::DeletedItem { time, .. } => time >= start_date,
                },
            })
        })
        .take(match options.limit {
            Some(i) => i,
            None => len,
        });

    let stdout = tokio::io::stdout();

    let mut writer = tokio::io::BufWriter::with_capacity(1024 * 1024, stdout);

    while let Some(item) = stream.next().await {
        let mut result = serde_json::to_string(&item)?;
        result.push('\n');
        writer.write(result.as_bytes()).await?;
    }
    writer.flush().await?;

    eprintln!("\nErrors: {}\n", errors);

    Ok(())
}

#[inline(always)]
async fn get_json<T: for<'de> serde::Deserialize<'de>>(
    client: &reqwest::Client,
    url: String,
) -> Result<T> {
    let resp = client.get(&url).send().await?;
    Ok(resp
        .json()
        .await
        .with_context(|| format!("fetching URL {}", url))?)
}
