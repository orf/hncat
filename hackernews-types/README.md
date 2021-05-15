# hackernews-types

This package contains simple types for interacting with the [Hacker News API](https://github.com/HackerNews/API). 
The API documentation is not great and there are some differences. The types contained within this crate can parse
every item returned from the hacker news API.


## Example:

```rust
use anyhow::Result;
use reqwest;
use hackernews_types::Item;

#[tokio::main]
async fn main() -> Result<()>{
    let client = reqwest::Client::new();
    let resp = client.get("https://hacker-news.firebaseio.com/v0/item/8863.json").send().await?;
    let item: Item = resp.json().await?;
    
    println!("Item: {:?}", item);
    
    Ok(())
}
```
