use std::path::Path;

use clap::Parser;

use scissors::{api, markdown, subprocess};

#[derive(Parser, Debug)]
struct Cli {
    /// Client ID
    #[arg(short, long)]
    id: i32,

    /// Client Secret
    #[arg(short, long)]
    secret: String,

    /// Files for checking. Either a list of paths, space-separated, or "auto"
    #[arg(short, long, use_value_delimiter = true, value_delimiter = ',')]
    files: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    let files = if !args.files.is_empty() {
        args.files
    } else {
        let branch = subprocess::git_oneline(&["branch", "--show-current"])
            .expect("git branch failed")
            .unwrap();
        if branch == "master" {
            panic!("please run the tool from a feature branch, or use \"--files path1,path2\"")
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

    let token =
        osu_api::get_client_token(args.id, &args.secret).expect("Failed to fetch guest API token");

    for filename in files {
        let path = Path::new(&filename);
        let article = markdown::Article::read(path).expect("failed to read the article");

        let mut all_mentions = vec![];
        let mut ids = vec![];
        article.get_user_profiles().drain().for_each(|mut e| {
            ids.push(e.0);
            all_mentions.append(&mut e.1);
        });
        ids.sort();
        all_mentions.sort_by_key(|m| (m.id.loc.line, m.id.loc.ch));

        let canonical_data = api::fetch_user_data(&token, &ids);
        let bad_mentions: Vec<&markdown::UserMention> = all_mentions
            .iter()
            .filter(|m| !canonical_data[&m.id.num].valid(m))
            .collect();
        if !bad_mentions.is_empty() {
            println!("--- {}: {} error(s)", filename, bad_mentions.len());
            for mention in bad_mentions {
                let user_data = &canonical_data[&mention.id.num];
                print!("\t{} (line {}):", user_data.username, mention.id.loc.line);
                if user_data.username != mention.username.text {
                    print!(
                        " wrong username (wanted: {}, got: {})",
                        user_data.username, mention.username.text
                    );
                }
                match &mention.country_code {
                    None => print!(" missing country code (wanted: {})", user_data.country_code),
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
    }
}
