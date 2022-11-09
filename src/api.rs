use std::{thread::sleep, time::Duration};

use serde::{Deserialize, Serialize};

// https://osu.ppy.sh/docs/index.html#get-users
const USER_LIMIT: usize = 50;
const THROTTLE_SLEEP_DURATION: Duration = Duration::from_secs(5);

#[derive(Debug, Serialize, Deserialize)]
struct UserResponse {
    users: Vec<super::User>,
}

pub fn fetch_user_data(token: &str, users: &[i32]) -> std::collections::HashMap<i32, super::User> {
    if users.len() > osu_api::REQUESTS_PER_MINUTE_LIMIT {
        eprintln!(
            "* More than {} users requested from API -- expect delays",
            osu_api::REQUESTS_PER_MINUTE_LIMIT
        )
    }

    let rl_checker = osu_api::new(token.to_owned());
    let get_rate_limit =
        || match osu_api::check_rate_limit(&rl_checker, "https://osu.ppy.sh/api/v2/users/672931") {
            Ok(rl) => rl,
            Err(e) => {
                eprintln!(
                    "* Failed to refresh API request limit, assuming default: {}",
                    e
                );
                osu_api::RateLimit {
                    limit: osu_api::REQUESTS_PER_MINUTE_LIMIT as i32,
                    remaining: 0,
                }
            }
        };

    let mut limit = get_rate_limit();
    let mut current_page = 0;
    let mut user_data = std::collections::HashMap::new();
    let mut threads: Vec<_> = vec![];
    loop {
        let start = current_page * USER_LIMIT;
        if start >= users.len() {
            break;
        }
        let end = std::cmp::min((current_page + 1) * USER_LIMIT, users.len() - 1);

        let chunk: Vec<String> = users[start..end].iter().map(|x| x.to_string()).collect();
        let url = format!(
            "https://osu.ppy.sh/api/v2/users?ids[]={}",
            chunk.join("&ids[]=")
        );
        let api = osu_api::new(token.to_owned());

        if limit.remaining <= USER_LIMIT as i32 {
            eprint!(
                "* Waiting for osu! API to stop throttling us ({}/{})",
                start,
                users.len()
            );
            while limit.remaining <= USER_LIMIT as i32 {
                // Sleep for extra requests allowed (usually 20/s),
                // provided they are replenished with sub-second frequency on the server side.
                eprint!(".");
                sleep(THROTTLE_SLEEP_DURATION);
                limit = get_rate_limit();
            }
            eprintln!();
        }

        let handle = std::thread::spawn(move || {
            api.get(url)
                .send()
                .and_then(|resp| resp.json::<UserResponse>().map(|payload| payload.users))
        });
        threads.push(handle);
        limit.remaining -= USER_LIMIT as i32; // Assuming that this call costs as much as there are users in the request
        current_page += 1;
    }

    threads.into_iter().for_each(|t| match t.join().unwrap() {
        Ok(users) => {
            user_data.extend(users.into_iter().map(|u| (u.id, u)));
        }
        Err(e) => {
            eprintln!("* Failed to query a batch of users: {}", e);
        }
    });
    user_data
}
