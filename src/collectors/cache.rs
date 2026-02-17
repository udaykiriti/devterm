use chrono::{DateTime, Utc};

use super::types::{AwsStatus, DashboardData, PrStatus};

pub fn apply_cache(data: &mut DashboardData, cache: &mut DataCache, cache_seconds: u64) {
    let now = Utc::now();

    match &data.aws.error {
        None => {
            cache.aws = Some((data.aws.clone(), now));
        }
        Some(err) => {
            if let Some((cached, ts)) = &cache.aws {
                if (now - *ts).num_seconds() <= cache_seconds as i64 {
                    data.aws.instances = cached.instances.clone();
                    data.aws.items = cached.items.clone();
                    data.aws.source = format!("{} (cached)", cached.source);
                    data.aws.error = Some(format!("{} | showing cached data", err));
                }
            }
        }
    }

    match &data.prs.error {
        None => {
            cache.prs = Some((data.prs.clone(), now));
        }
        Some(err) => {
            if let Some((cached, ts)) = &cache.prs {
                if (now - *ts).num_seconds() <= cache_seconds as i64 {
                    data.prs.open = cached.open.clone();
                    data.prs.items = cached.items.clone();
                    data.prs.source = format!("{} (cached)", cached.source);
                    data.prs.error = Some(format!("{} | showing cached data", err));
                }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct DataCache {
    aws: Option<(AwsStatus, DateTime<Utc>)>,
    prs: Option<(PrStatus, DateTime<Utc>)>,
}
