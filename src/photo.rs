use chrono::NaiveDateTime;
use std::convert::AsRef;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Photo {
    date: Option<NaiveDateTime>,
    path: Option<String>,
    new_path: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct PhotoBuilder {
    photo: Photo,
}

impl Photo {
    pub fn new() -> Photo {
        return Photo {
            date: None,
            path: None,
            new_path: None,
        };
    }

    pub fn from(path: String, date: NaiveDateTime) -> Photo {
        return Photo {
            date: Option::from(date),
            path: Option::from(path),
            new_path: None,
        };
    }

    pub fn set_date(&mut self, date: NaiveDateTime) {
        self.date = Option::from(date);
    }

    pub fn date(&self) -> Option<NaiveDateTime> {
        return self.date;
    }

    pub fn set_path(&mut self, path: String) {
        self.path = Option::from(path);
    }

    pub fn path(&self) -> &Option<String> {
        return &self.path;
    }

    pub fn set_new_path(&mut self, new_path: String) {
        self.new_path = Option::from(new_path);
    }

    pub fn new_path(&self) -> &Option<String> {
        return &self.new_path;
    }
}

impl PhotoBuilder {
    pub fn new() -> PhotoBuilder {
        return PhotoBuilder {
            photo: Photo {
                date: None,
                path: None,
                new_path: None,
            },
        };
    }

    pub fn with_date(&mut self, date: NaiveDateTime) -> &mut PhotoBuilder {
        self.photo.set_date(date);
        return self;
    }

    pub fn with_path(&mut self, path: String) -> &mut PhotoBuilder {
        self.photo.set_path(path);
        return self;
    }

    pub fn with_new_path(&mut self, new_path: String) -> &mut PhotoBuilder {
        self.photo.set_new_path(new_path);
        return self;
    }

    pub fn build(&self) -> Photo {
        self.photo.clone()
    }
}
