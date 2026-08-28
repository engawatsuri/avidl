use std::env;
use std::fs;
use std::process::Command;
use chrono::Local;
use chrono::TimeDelta;
use chrono::DateTime;
use chrono::Datelike;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono::Utc;
use chrono::Weekday;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url_tpls: [(i32, &str); 15] = [
        (1, "http://radiko.jp/#!/ts/TBS/{}000000"), // 空気階段の踊り場
        (1, "http://radiko.jp/#!/ts/TBS/{}010000"), // JUNK 伊集院光 深夜の馬鹿力
        (2, "http://radiko.jp/#!/ts/TBS/{}000000"), // アルコ&ピース D.C.GARAGE
        (2, "http://radiko.jp/#!/ts/TBS/{}010000"), // JUNK 爆笑問題カーボーイ
        (2, "http://radiko.jp/#!/ts/KBS/{}193000"), // 角田龍平の蛤御門のヘン
        (2, "http://radiko.jp/#!/ts/TBS/{}220000"), // 佐倉綾音 論理×ロンリー
        (3, "http://radiko.jp/#!/ts/TBS/{}010000"), // 山里亮太の不毛な議論
        (3, "http://radiko.jp/#!/ts/LFR/{}030000"), // 佐久間宣行のオールナイトニッポン0(ZERO)
        (4, "http://radiko.jp/#!/ts/TBS/{}000000"), // ハライチのターン！
        (4, "http://radiko.jp/#!/ts/LFR/{}010000"), // ナインティナインのオールナイトニッポン
        (4, "https://www.youtube.com/playlist?list=PLxPYUI5vtD6TNW32Yjl2dYirqIaPJSpYL"), // ドラえもん
        (4, "https://www.youtube.com/playlist?list=PLr8TEuYjnMZXaq-aPICgBDFgck3BQP_JK"), // 日常組
        (5, "http://radiko.jp/#!/ts/OBC/{}230000"), // 森久保祥太郎・浪川大輔　つまみは塩だけ
        (5, "https://www.youtube.com/playlist?list=PLxPYUI5vtD6TNW32Yjl2dYirqIaPJSpYL"), // ドラえもん
        (6, "http://radiko.jp/#!/ts/LFR/{}010000"), // オードリーのオールナイトニッポン
    ];
    let mut urls: Vec<String> = Vec::new();
    let args: Vec<String> = env::args().collect();
    let today = Local::now();
    let weekday = today.weekday().num_days_from_monday() as i32;
    let before = if args.len() >= 2 && let Ok(before_weekday) = args[1].parse::<Weekday>() {
        (weekday - before_weekday.num_days_from_monday() as i32 + 7) % 7
    } else {
        1
    };

    for url_tpl in url_tpls {
        let lastest_delta = (weekday - url_tpl.0 + 7) % 7;
        if lastest_delta < 7 && lastest_delta <= before {
            if url_tpl.1.starts_with("http://radiko.jp") {
                let lastest = today - TimeDelta::days(lastest_delta as i64);
                urls.push(url_tpl.1.to_string().replace("{}", &lastest.format("%Y%m%d").to_string()));
            } else if url_tpl.1.starts_with("https://www.youtube.com") {
                let response = ureq::get(url_tpl.1).call()?.into_reader();
                let feed = feed_rs::parser::parse(response)?;
                let start_naive = NaiveDateTime::new(
                    today.date_naive() - Duration::days(lastest_delta as i64),
                    chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                );
                let start: DateTime<Utc> = Local.from_local_datetime(&start_naive).unwrap().with_timezone(&Utc);
                for entry in feed.entries {
                    if let Some(time) = entry.published {
                        if start <= time && time < start + Duration::days(1) {
                            if let Some(link) = entry.links.first() {
                                urls.push(link.href);
                                break;
                            }
                        }
                    }
                }
            } else {
                todo!();
            }
        }
    }

    println!("number of downloading: {}", urls.len());
    for url in &urls {
        let output = if url.starts_with("http://radiko.jp") {
            Command::new("yt-dlp").arg(&url).output()
        } else {
            Command::new("yt-dlp").args(["-f", "134+139", &url]).output()
        };
        match output {
            Ok(out) if out.status.success() => {
                println!("success: {}", url);
            }
            Ok(_out) => {
                eprintln!("failure: {}", url);
            }
            Err(_e) => {
                eprintln!("failure: yt-dlp");
                let mut content = String::new();
                for url in &urls {
                    content.push_str(&url);
                    content.push('\n');
                }
                let _ = fs::write("downloading", &content);
                return Ok(());
            }
        }
    }

    Ok(())
}
