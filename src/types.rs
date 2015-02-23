use time::{Tm};

#[derive(Clone)]
pub struct Hit {
    pub host: String,
    pub time: Tm,
    pub status: usize,
    pub path: String,
    pub referer: String,
    pub agent: String,
}

#[derive(Clone)]
pub struct Visit {
    pub host: String,
    pub hit_count: usize,
    pub first_hit_time: Tm,
    pub last_hit_time: Tm,
    pub last_path: String,
    pub referer: String,
    pub agent: String,
}

