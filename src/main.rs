mod error;
mod login;

use error::{Error, ErrorKind};
use failure::ResultExt;
use html_extractor::*;

type Result<T> = std::result::Result<T, Error>;

macro_rules! print_f {
    ($($tt:tt)*) => {{
        print!($($tt)*);
        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }};
}

#[tokio::main]
async fn main() {
    if let Err(err) = init_data_dir() {
        eprintln!("Error: {}", err);
    }

    use clap::*;
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("opt_login_id").short("l").takes_value(true))
        .arg(Arg::with_name("opt_password").short("p").takes_value(true))
        .subcommand(
            SubCommand::with_name("login")
                .arg(Arg::with_name("login_id").required(true))
                .arg(Arg::with_name("password").required(true)),
        )
        .subcommand(SubCommand::with_name("logout"));
    let matches = app.get_matches();
    match matches.subcommand() {
        ("login", Some(args)) => {
            if let Err(err) = login(
                args.value_of("login_id").unwrap(),
                args.value_of("password").unwrap(),
            ) {
                eprintln!("Error: {}", err);
            }
        }
        ("logout", _) => {
            if let Err(err) = logout() {
                eprintln!("Error: {}", err);
            }
        }
        _ => {
            let login_info = if let (Some(id), Some(pw)) = (
                matches.value_of("opt_login_id"),
                matches.value_of("opt_password"),
            ) {
                Some(login::LoginInfo {
                    login_id: id.to_owned(),
                    password: pw.to_owned(),
                })
            } else {
                None
            };
            if let Err(err) = check(login_info).await {
                eprintln!("Error: {}", err);
            }
        }
    }
}

fn init_data_dir() -> Result<()> {
    let data_dir = dirs::home_dir()
        .unwrap()
        .join(".waseda-moodle-checker")
        .join("courses");
    std::fs::create_dir_all(data_dir)?;

    Ok(())
}

fn login(login_id: &str, password: &str) -> Result<()> {
    let login_info = login::LoginInfo {
        login_id: login_id.to_owned(),
        password: password.to_owned(),
    };
    login_info.save()?;
    println!("successfully saved login info");
    Ok(())
}
fn logout() -> Result<()> {
    let login_info_file = dirs::home_dir()
        .unwrap()
        .join(".waseda-moodle-checker")
        .join("login_info.json");
    std::fs::remove_file(login_info_file)?;
    Ok(())
}

async fn check(login_info: Option<login::LoginInfo>) -> Result<()> {
    let login_info = if let Some(l) = login_info {
        l
    } else {
        login::LoginInfo::load()?
    };

    print_f!("logging in...");
    let session = waseda_moodle::Session::login(&login_info.login_id, &login_info.password)
        .await
        .context(ErrorKind::LoginError)?;
    println!("ok");

    print_f!("fetching courses list...");
    let list = waseda_moodle::course::fetch_enrolled_courses(&session)
        .await
        .context(ErrorKind::InvalidResponse)?;
    println!("ok");

    let mut first_fetched = Vec::new();
    let mut updated = Vec::new();

    let mut count = 0;
    let total = list.len();
    for c in list {
        count += 1;
        let c_status = check_course(&session, &c, count, total).await?;

        match c_status {
            Status::Updated => updated.push(c),
            Status::FirstFetched => first_fetched.push(c),
            _ => (),
        }
    }

    print_f!("\n");

    if first_fetched.len() > 0 || updated.len() > 0 {
        if first_fetched.len() > 0 {
            println!("First fetched:");
            for c in first_fetched {
                println!("\t{} - {}", c.name, c.view_url);
            }
        }
        if updated.len() > 0 {
            println!("\nUpdated:");
            for c in updated {
                println!("\t{} - {}", c.name, c.view_url);
            }
        }
    } else {
        println!("There are no updates.")
    }

    Ok(())
}
async fn check_course(
    session: &waseda_moodle::Session,
    course: &waseda_moodle::course::Course,
    count: usize,
    total: usize,
) -> Result<Status> {
    print_f!(
        "fetching course page ({}/{}) id={} '{}'...",
        count,
        total,
        course.id,
        course.name
    );
    let course_page = to_comparable_object(
        &session
            .client
            .get(&course.view_url)
            .send()
            .await?
            .text()
            .await?,
        session,
    )?;
    print_f!("ok ");

    let downloaded_course_page = dirs::home_dir()
        .unwrap()
        .join(".waseda-moodle-checker")
        .join("courses")
        .join(format!("{}.html", course.id));

    let status = if downloaded_course_page.exists() {
        use std::io::Read;
        let mut prev_course_page_str = String::new();
        std::fs::OpenOptions::new()
            .read(true)
            .open(&downloaded_course_page)?
            .read_to_string(&mut prev_course_page_str)?;
        let prev_course_page = scraper::Html::parse_document(&prev_course_page_str);

        if prev_course_page == course_page {
            Status::NoUpdates
        } else {
            Status::Updated
        }
    } else {
        Status::FirstFetched
    };

    if status != Status::NoUpdates {
        use std::io::Write;
        let mut writer = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(downloaded_course_page)?;
        write!(writer, "{}", course_page.root_element().html())?;
    }

    println!("(status={:?})", status);

    Ok(status)
}

#[derive(Debug, PartialEq)]
enum Status {
    NoUpdates,
    FirstFetched,
    Updated,
}
html_extractor! {
    CoursePage {
        content: String = (inner_html of "#page-content"),
    }
}

fn to_comparable_object(raw_html: &str, session: &waseda_moodle::Session) -> Result<scraper::Html> {
    lazy_static::lazy_static! {
        static ref REGEX: regex::Regex = regex::Regex::new(r#"single_button[^"]*""#).unwrap();
    }
    Ok(scraper::Html::parse_document(
        &CoursePage::extract_from_str(&REGEX.replace_all(
            &raw_html.replace(&session.session_key, ""),
            r#"single_button""#,
        ))?
        .content,
    ))
}
