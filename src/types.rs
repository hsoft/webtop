extern crate time;

use time::{Tm};

#[deriving(Clone)]
pub struct Hit {
    pub host: String,
    pub time: Tm,
    pub status: uint,
    pub path: String,
    pub referer: String,
    pub agent: String,
}

#[deriving(Clone)]
pub struct Visit {
    pub host: String,
    pub hit_count: uint,
    pub first_hit_time: Tm,
    pub last_hit_time: Tm,
    pub last_path: String,
    pub referer: String,
    pub agent: String,
}

