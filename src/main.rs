use std::env;
use std::fs;
use std::process::Command;
use chrono::Local;
use chrono::TimeDelta;
use chrono::Datelike;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url_tpls: [(i32, &str); 12] = [
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
        (5, "http://radiko.jp/#!/ts/OBC/{}230000"), // 森久保祥太郎・浪川大輔　つまみは塩だけ
        (6, "http://radiko.jp/#!/ts/LFR/{}010000"), // オードリーのオールナイトニッポン
    ];
    let mut urls: Vec<String> = Vec::new();
    let args: Vec<String> = env::args().collect();
    let today = Local::now();
    let (weekday, before) = if args.len() >= 2 {
        let weekday = today.weekday().num_days_from_monday() as i32;
        (weekday, (weekday - args[1].parse::<i32>()? + 7) % 7)
    } else {
        ((today.weekday().num_days_from_monday() as i32 - 1 + 7) % 7, 1)
    };

    for url_tpl in url_tpls {
        let lastest_delta = if weekday == url_tpl.0 {
            let url_time = url_tpl.1[30..].parse::<i32>()?;
            let today_time = today.format("%H%M%S").to_string().parse::<i32>()?;
            if url_time < today_time {
                continue;
            } else {
                7
            }
        } else {
            (weekday - url_tpl.0 + 7) % 7
        };
        if lastest_delta <= before {
            let lastest = today - TimeDelta::days(lastest_delta as i64);
            urls.push(url_tpl.1.to_string().replace("{}", &lastest.format("%Y%m%d").to_string()));
        }
    }

    println!("number of downloading: {}", urls.len());
    for url in &urls {
        let output = Command::new("yt-dlp").arg(&url).output();
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
