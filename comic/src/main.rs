use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use futures::stream::{self, StreamExt};
use reqwest::Client;

async fn download_chapter(client: Client, m: String, ch: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dir_name = format!("{:03}", ch);
    let dir_path = Path::new(&dir_name);
    let mut format = 0;

    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }

    let mut origin_html;
    let mut img = 0;

    let origin = client.get(format!("https://mangarawad.city/manga/{}/chapter-{}", m, ch)).send().await?;
    if origin.status().is_success() {
        match origin.text().await {
            Ok(html) => {
                origin_html = html;
            }
            Err(_) => {
                println!("{}: internal: no origin", ch);
                return Ok(());
            }
        }
    } else {
        if img == 0 {
            println!("{}: internal: no origin", ch);
        }
        return Ok(());
    }

    
    let target = vec![
        ("https://iphotomg.com/{}/chapter_{}/page_{}.jpg", "jpg"),
        ("https://iphotomg.com/{}/chapter_{}/page_{}.webp", "webp"),
        ("https://iphotomg.com/{}/chapter_{}/image_{}.webp", "webp"),
        ("https://iphotomg.com/{}/chapter_{}/{}.webp", "webp"),
    ];
    for (t, _ex) in target.iter() {
        let url = t.replacen("{}", &m, 1).replacen("{}", &ch.to_string(), 1).replacen("{}", &img.to_string(), 1);
        if origin_html.contains(&url) {
            break;
        }
        format += 1;
    }
    if format >= target.len() {
        println!("error: no match format");
        return Ok(());
    }
    loop {
        let url = target[format].0.replacen("{}", &m, 1).replacen("{}", &ch.to_string(), 1).replacen("{}", &img.to_string(), 1);
        let file_name = format!("{:02}.{}", img, target[format].1);
        let out_file = dir_path.join(file_name);
        if !origin_html.contains(&url) {
            if img == 0 {
                println!("{}: internal: not found", ch);
            } else {
                println!("{}: {}", ch, img);
            }
            break;
        }
        match client.get(&url).send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    if img == 0 {
                        println!("{}: internal: {}", ch, response.status());
                    }
                    break; 
                }

                if let Ok(bytes) = response.bytes().await {
                    match image::load_from_memory(&bytes) {
                        Ok(_) => {
                            let mut file = File::create(out_file)?;
                            file.write_all(&bytes)?;
                        }
                        Err(_) => {
                        }
                    }
                }
            }
            Err(_) => {
            }
        }

        img += 1;

        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("error: no match cmd");
        return Ok(());
    }

    // 共有して使うHTTPクライアントを作成
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let concurrent_requests = 10;

    println!("start (並行数: {})...", concurrent_requests);

    stream::iter(args[2].parse().unwrap()..=args[3].parse().unwrap()).for_each_concurrent(concurrent_requests, |ch| {
        let client_clone = client.clone();
        let m_clone = args[1].clone();
        async move {
            if let Err(e) = download_chapter(client_clone, m_clone, ch).await {
                eprintln!("{}: external: {}", ch, e);
            }
        }
    }).await;

    println!("finish");
    Ok(())
}
