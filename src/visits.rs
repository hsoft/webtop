use std::collections::hash_map::{HashMap, Entry};
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

type VisitID = u32;
pub type VisitHolder = HashMap<VisitID, Box<Visit>>;
pub type HostVisitMap = HashMap<String, VisitID>;
pub type PathVisitMap = HashMap<String, Box<HashSet<VisitID>>>;

pub struct VisitStats {
    visit_counter: u32,
    last_seen_time: ::time::Tm,
    pub visits: VisitHolder,
    pub host_visit_map: HostVisitMap,
    pub path_visit_map: PathVisitMap,
}

impl VisitStats {
    pub fn new() -> VisitStats {
        VisitStats {
            visit_counter: 0,
            last_seen_time:  ::time::now(),
            visits: HashMap::new(),
            host_visit_map: HashMap::new(),
            path_visit_map: HashMap::new(),
        }
    }

    pub fn feed_hit(&mut self, hit: &Hit) {
        let key = &hit.host;
        let visitid: VisitID = match self.host_visit_map.entry(key.clone()) {
            Entry::Occupied(e) => {
                *e.get()
            }
            Entry::Vacant(e) => {
                self.visit_counter += 1;
                let visitid = self.visit_counter;
                let visit = Box::new(Visit {
                    host: hit.host.clone(),
                    hit_count: 0,
                    first_hit_time: hit.time,
                    last_hit_time: hit.time,
                    last_path: hit.path.clone(),
                    referer: hit.referer.clone(),
                    agent: hit.agent.clone(),
                });
                self.visits.insert(visitid, visit);
                e.insert(visitid);
                visitid
            }
        };
        let visit: &mut Box<Visit> = self.visits.get_mut(&visitid).unwrap();
        visit.hit_count += 1;
        visit.last_hit_time = hit.time;
        visit.last_path = hit.path.clone();
        self.last_seen_time = hit.time;
        let key = &hit.path;
        match self.path_visit_map.entry(key.clone()) {
            Entry::Occupied(e) => {
                let visits: &mut Box<HashSet<u32>> = e.into_mut();
                visits.insert(visitid);
            }
            Entry::Vacant(e) => {
                let mut visits = Box::new(HashSet::new());
                visits.insert(visitid);
                e.insert(visits);
            }
        };
    }

    pub fn purge_visits(&mut self) {
        let mut toremove: Vec<u32> = Vec::new();
        let last_seen_ts = self.last_seen_time.to_timespec();
        for (visitid, visit) in self.visits.iter() {
            if last_seen_ts.sec - visit.last_hit_time.to_timespec().sec > 5 * 60 {
                toremove.push(*visitid);
                self.host_visit_map.remove(&visit.host);
            }
        }
        for visitid in toremove.iter() {
            self.visits.remove(visitid);
        }
    }

}
