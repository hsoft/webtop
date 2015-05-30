use time::strftime;

#[derive(Clone)]
pub struct Hit {
    pub host: String,
    pub time: ::time::Tm,
    pub status: u32,
    pub path: String,
    pub referer: String,
    pub agent: String,
}

impl Hit {
    pub fn is_4xx(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    pub fn is_5xx(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    pub fn fmt_time(&self) -> String {
        strftime("%H:%M", &self.time).unwrap()
    }

}

