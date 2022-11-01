use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use clap::{Parser, Subcommand};

use scissors::{api, markdown, subprocess};

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    /// Type of test to perform
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug, Clone)]
enum Action {
    Users {
        /// OAuth app ID
        #[arg(short, long)]
        id: i32,

        /// OAuth app secret
        #[arg(short, long)]
        secret: String,

        /// Files to check (pass them as regular arguments, not flags). If omitted, use `git diff`
        files: Vec<String>,

        /// Detect flags that are missing near user profiles (off by default)
        #[arg(short, long)]
        flags: bool,

        /// Detect username mismatches (off by default)
        #[arg(short, long)]
        names: bool,
    },
}

fn test_users(
    id: i32,
    secret: &str,
    files: Vec<String>,
    country_required: bool,
    name_required: bool,
) {
    let files = if !files.is_empty() {
        files
    } else {
        let branch = subprocess::git_oneline(&["branch", "--show-current"])
            .expect("git branch failed")
            .unwrap();
        if branch == "master" {
            panic!("please run the tool from a feature branch, or pass file paths as command line arguments")
        }

        let first_commit = subprocess::git_oneline(&[
            "log",
            format!("master..{}", branch).as_str(),
            "--pretty=format:%H",
        ])
        .expect("git log failed")
        .unwrap_or_else(|| "HEAD".to_owned());

        subprocess::git(
            &[
                "diff",
                "--no-renames",
                "--name-only",
                "--diff-filter=d",
                format!("{}^", first_commit).as_str(),
            ],
            subprocess::OutputLines::All,
        )
        .expect("git diff failed")
        .into_iter()
        .filter(|filename| filename.ends_with(".md"))
        .collect()
    };

    let token = osu_api::get_client_token(id, secret).expect("Failed to fetch guest API token");
    let mut contents = HashMap::<&String, Vec<markdown::UserMention>>::new();
    let mut user_ids = HashSet::<i32>::new();

    for filename in &files {
        let article =
            markdown::Article::read(Path::new(&filename)).expect("failed to read the article");
        let mut all_mentions = vec![];
        article.get_user_profiles().into_iter().for_each(|mut e| {
            user_ids.insert(e.0);
            all_mentions.append(&mut e.1);
        });
        all_mentions.sort_by_key(|m| (m.id.loc.line, m.id.loc.ch));
        contents.insert(filename, all_mentions);
    }

    let user_ids = user_ids.into_iter().collect::<Vec<i32>>();
    let canonical_data = api::fetch_user_data(&token, &user_ids);

    for filename in &files {
        let bad_mentions: Vec<&markdown::UserMention> = contents[filename]
            .iter()
            .filter(|mention| {
                // filter out restricted users (API returns no data for them)
                canonical_data.get(&mention.id.num).map_or(false, |m| {
                    !m.valid(mention, country_required, name_required)
                })
            })
            .collect();

        if bad_mentions.is_empty() {
            continue;
        }

        println!("--- {}: {} error(s)", filename, bad_mentions.len());
        for mention in bad_mentions {
            let user_data = &canonical_data[&mention.id.num];
            print!("\t{} (line {}):", user_data.username, mention.id.loc.line);
            if name_required && user_data.username != mention.username.text {
                print!(
                    " wrong username (wanted: {}, got: {})",
                    user_data.username, mention.username.text
                );
            }
            match &mention.country_code {
                None => {
                    if country_required {
                        print!(" missing country code (wanted: {})", user_data.country_code)
                    }
                }
                Some(country_code) => {
                    if country_code.text != user_data.country_code {
                        print!(
                            " wrong country code (wanted: {}, got: {})",
                            user_data.country_code, country_code.text
                        );
                    }
                }
            }
            println!();
        }
        println!();
    }
    println!(
        "Checked {} mention(s) in {} file(s)",
        contents.values().map(|m| m.len()).sum::<usize>(),
        files.len()
    )
}

fn main() {
    let args = Cli::parse();

    match args.action {
        Action::Users {
            id,
            secret,
            files,
            flags,
            names,
        } => test_users(id, &secret, files, flags, names),
    }
}
