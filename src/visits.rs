use std::cmp::Ordering;
use std::collections::hash_map;
use std::collections::hash_set::HashSet;
use std::vec;

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
type VisitHolder = hash_map::HashMap<VisitID, Box<Visit>>;
type HostVisitMap = hash_map::HashMap<String, VisitID>;
type PathVisitMap = hash_map::HashMap<String, Box<HashSet<VisitID>>>;

pub struct VisitStats {
    visit_counter: u32,
    last_seen_time: ::time::Tm,
    visits: VisitHolder,
    host_visit_map: HostVisitMap,
    path_visit_map: PathVisitMap,
}

impl VisitStats {
    pub fn new() -> VisitStats {
        VisitStats {
            visit_counter: 0,
            last_seen_time:  ::time::now(),
            visits: hash_map::HashMap::new(),
            host_visit_map: hash_map::HashMap::new(),
            path_visit_map: hash_map::HashMap::new(),
        }
    }

    pub fn feed_hit(&mut self, hit: &Hit) {
        let key = &hit.host;
        let visitid: VisitID = match self.host_visit_map.entry(key.clone()) {
            hash_map::Entry::Occupied(e) => {
                *e.get()
            }
            hash_map::Entry::Vacant(e) => {
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
            hash_map::Entry::Occupied(e) => {
                let visits: &mut Box<HashSet<u32>> = e.into_mut();
                visits.insert(visitid);
            }
            hash_map::Entry::Vacant(e) => {
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

    pub fn visit_count(&self) -> usize {
        self.visits.len()
    }

    pub fn iter_sorted_visits(&self) -> vec::IntoIter<&Box<Visit>> {
        let mut sorted_visits: Vec<&Box<Visit>> = self.visits.values().collect();
        sorted_visits.sort_by(
            |a, b| match (&a.hit_count).cmp(&b.hit_count).reverse() {
                Ordering::Equal => a.last_hit_time.to_timespec().cmp(&b.last_hit_time.to_timespec()).reverse(),
                x => x,
            }
        );
        sorted_visits.into_iter()
    }

    pub fn iter_sorted_path_chunks(&self) -> vec::IntoIter<(&str, usize)> {
        let mut sorted_path_chunks: Vec<(&str, usize)> = self.path_visit_map.iter().map(
            |(key, value)| (&key[..], value.len())
        ).collect();
        sorted_path_chunks.sort_by(
            |a, b| a.1.cmp(&b.1).reverse()
        );
        sorted_path_chunks.into_iter()
    }
}

