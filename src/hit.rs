use time::strftime;

const PAGE_EXTS: [&'static str; 3] = ["html", "htm", "php"];

#[derive(Clone)]
pub struct Hit {
    pub host: String,
    pub time: ::time::Tm,
    pub status: u32,
    pub bytes: u32,
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

    pub fn is_resource(&self) -> bool {
        is_path_resource(&self.path)
    }
}

/// Returns whether `path` is a "resource hit".
///
/// A resource hit is a hit that wasn't requested directly by the client, but rather is
/// a requirement of a previous hit. Typically, images, CSS, JS are considered "resource hits".
///
pub fn is_path_resource(path: &str) -> bool {
    let last_elem = path.rsplitn(2, '/').next().unwrap();
    let ext = last_elem.rsplitn(2, '.').next().unwrap();
    if ext.len() == 0 {
        return false;
    }
    !PAGE_EXTS.contains(&ext)
}

