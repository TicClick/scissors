use serde::{Deserialize, Serialize};

// https://osu.ppy.sh/docs/index.html#get-users
const USER_LIMIT: usize = 50;

#[derive(Debug, Serialize, Deserialize)]
struct UserResponse {
    users: Vec<super::User>,
}

pub fn fetch_user_data(token: &str, users: &[i32]) -> std::collections::HashMap<i32, super::User> {
    let mut user_data = std::collections::HashMap::new();
    let mut threads: Vec<_> = vec![];

    for batch in users.chunks(USER_LIMIT) {
        let ids: Vec<String> = batch.iter().map(|x| x.to_string()).collect();
        let url = format!(
            "https://osu.ppy.sh/api/v2/users?ids[]={}",
            ids.join("&ids[]=")
        );
        let api = osu_api::new(token.to_owned());

        let handle = std::thread::spawn(move || {
            api.get(url)
                .send()
                .and_then(|resp| resp.json::<UserResponse>().map(|payload| payload.users))
        });
        threads.push(handle);
    }

    threads.into_iter().for_each(|t| match t.join().unwrap() {
        Ok(users) => {
            user_data.extend(users.into_iter().map(|u| (u.id, u)));
        }
        Err(e) => {
            eprintln!("Failed to query a batch of users: {}", e);
        }
    });
    user_data
}
