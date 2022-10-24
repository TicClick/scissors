use std::collections::HashMap;
use std::{fs, io, path};

use regex::Regex;

#[derive(Debug)]
pub struct Article<'a> {
    pub path: Box<&'a path::Path>,
    pub lines: Vec<String>,
}

impl<'a> Article<'a> {
    pub fn read(p: &'a path::Path) -> Result<Self, io::Error> {
        p.try_exists().and_then(|status| {
            if !status {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Broken symbolic link?",
                ))
            } else {
                fs::read_to_string(p).map(|payload| Self {
                    path: Box::new(p),
                    lines: payload.lines().map(|s| s.to_owned()).collect(),
                })
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub line: i32,
    pub ch: i32,
}

#[derive(Debug, Clone)]
pub struct UserId {
    pub num: i32,
    pub loc: Location,
}

#[derive(Debug)]
pub struct UserCountry {
    /// upper_char & lower_char
    pub text: String,
    pub loc: Location,
}

#[derive(Debug)]
pub struct UserName {
    pub text: String,
    pub loc: Location,
}

#[derive(Debug)]
pub struct UserMention {
    pub username: UserName,
    pub id: UserId,
    pub country_code: Option<UserCountry>,
}

fn canonize_username(s: &str) -> String {
    s.chars().filter(|ch| *ch != '*' && *ch != '\\').collect()
}

impl UserMention {
    pub fn from_matches(
        user: &regex::Match,
        user_id: &regex::Match,
        country_code: &Option<regex::Match>,
        line: i32,
    ) -> Self {
        Self {
            username: UserName {
                text: canonize_username(user.as_str()),
                loc: Location {
                    line,
                    ch: user.start() as i32,
                },
            },
            id: UserId {
                num: user_id.as_str().parse::<i32>().unwrap(),
                loc: Location {
                    line,
                    ch: user_id.start() as i32,
                },
            },
            country_code: country_code.as_ref().map(|match_| UserCountry {
                text: match_.as_str().to_owned(),
                loc: Location {
                    line,
                    ch: match_.start() as i32,
                },
            }),
        }
    }
}

impl Article<'_> {
    pub fn regex() -> Regex {
        Regex::new(
            r"(?x)
        (?P<user_flag>
            ::\{\ flag=(?P<country_code>[a-zA-Z]{2})\ \}::
        )??\ +?
        \[
            (?P<username_dirty>
                .+?
            )
        \]
        \(
            https://osu.ppy.sh/users/
            (?P<user_id>
                \d+
            )
        \)",
        )
        .unwrap()
    }

    pub fn get_user_profiles(&self) -> HashMap<i32, Vec<UserMention>> {
        let re = Self::regex();
        let mut kv: HashMap<i32, Vec<UserMention>> = HashMap::new();

        for (i, line) in self.lines.iter().enumerate() {
            for caps in re.captures_iter(line) {
                let username = &caps
                    .name("username_dirty")
                    .expect("No username block detected");
                let user_id = &caps
                    .name("user_id")
                    .expect("No profile link block detected");
                let user_country = &caps.name("country_code");
                let mention = UserMention::from_matches(username, user_id, user_country, i as i32);
                kv.entry(mention.id.num).or_default().push(mention);
            }
        }
        kv
    }
}
