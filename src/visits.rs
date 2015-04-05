use std::collections::hash_map::{HashMap};
use std::collections::hash_set::HashSet;

#[derive(Clone)]
pub struct Hit {
    pub host: String,
    pub time: ::time::Tm,
    pub status: u32,
    pub path: String,
    pub referer: String,
    pub agent: String,
}

#[derive(Clone)]
pub struct Visit {
    pub host: String,
    pub hit_count: u32,
    pub first_hit_time: ::time::Tm,
    pub last_hit_time: ::time::Tm,
    pub last_path: String,
    pub referer: String,
    pub agent: String,
}

pub type VisitID = u32;
pub type VisitHolder = HashMap<VisitID, Box<Visit>>;
pub type HostVisitMap = HashMap<String, VisitID>;
pub type PathVisitMap = HashMap<String, Box<HashSet<VisitID>>>;

pub struct VisitStats {
    pub visits: VisitHolder,
    pub visit_counter: u32,
    pub host_visit_map: HostVisitMap,
    pub path_visit_map: PathVisitMap,
    pub last_seen_time: ::time::Tm,
}

impl VisitStats {
    pub fn new() -> VisitStats {
        VisitStats {
            visits: HashMap::new(),
            visit_counter: 0,
            host_visit_map: HashMap::new(),
            path_visit_map: HashMap::new(),
            last_seen_time:  ::time::now(),
        }
    }
}
