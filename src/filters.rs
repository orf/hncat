use anyhow::Result;
use structopt::clap::ArgGroup;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("date_range").multiple(true).conflicts_with_all(& ["id_range", "relative_range"]))]
pub struct DateRangeFilter {
    /// Ignore all items created before this date. Supports relative durations like "1 hour" as well
    /// as absolute rfc3339 dates.
    #[structopt(long, group = "date_range", parse(try_from_str=parse_duration))]
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Ignore all items created after this date. Supports relative durations like "1 hour" as well
    /// as absolute rfc3339 dates.
    #[structopt(long, group = "date_range", parse(try_from_str=parse_duration))]
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("relative_range").conflicts_with_all(& ["id_range", "date_range"]))]
pub struct RelativeRangeFilter {
    /// Fetch all records since this time. Supports relative durations like "1 hour".
    #[structopt(long, group = "relative_range", parse(try_from_str=parse_duration))]
    pub since: Option<chrono::DateTime<chrono::Utc>>,

    /// Fetch the last X items.
    #[structopt(long, group = "relative_range")]
    pub last: Option<u32>,
}

#[derive(Debug, StructOpt)]
#[structopt(group = ArgGroup::with_name("id_range").multiple(true).conflicts_with_all(& ["date_range", "relative_range"]))]
pub struct IDRangeFilter {
    /// Fetch records with IDs higher than this.
    #[structopt(long, group = "id_range")]
    pub start_id: Option<u32>,

    /// Fetch records with IDs lower than this.
    #[structopt(long, group = "id_range")]
    pub stop_id: Option<u32>,
}

pub fn parse_duration(value: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    if let Ok(dt) = humantime::parse_rfc3339_weak(value) {
        return Ok(chrono::DateTime::from(dt));
    }

    let duration = humantime::parse_duration(value)?;
    let now = chrono::Utc::now();
    Ok(now - chrono::Duration::from_std(duration)?)
}
