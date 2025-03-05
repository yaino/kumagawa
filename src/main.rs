use dotenvy::dotenv;
use futures_util::TryStreamExt;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, AUTHORIZATION, CONTENT_TYPE,
};
use reqwest::{Client, StatusCode};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Duration;
use tokio::time::sleep;

mod schemas;
use crate::schemas::create_music::{
    Attributes, CreateMusicRequest, CreateMusicResponse, Prompt, RequestParams,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let auth_key = env::var("STABLE_AUDIO_AUTH_KEY").expect("STABLE_AUDIO_AUTH_KEY is not set");
    let auth_key = format!("Bearer {}", auth_key);
    let weight: u32 = env::var("STABLE_AUDIO_WEIGHT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| 1);
    let length_seconds: u32 = env::var("STABLE_AUDIO_LENGTH_SECS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| 180);
    let seed: u32 = env::var("STABLE_AUDIO_SEED")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| 123);
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("使い方: cargo run -- <引数>  # 引数がpromptになる");
        return Ok(());
    }
    let prompt = args.get(1).expect("can't read prompt").clone();

    let download_music_url = generate_music(
        prompt,
        weight,
        length_seconds,
        seed,
        auth_key.as_str(),
    )
    .await?;
    download_file(download_music_url, auth_key.as_str()).await?;
    Ok(())
}

async fn generate_music(
    prompt: String,
    weight: u32,
    length_seconds: u32,
    seed: u32,
    auth_key: &str,
) -> Result<String, reqwest::Error> {
    // クライアントを生成
    let client = Client::new();

    // ヘッダーを作成
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(auth_key).unwrap());

    // 送信するJSONデータ
    let request_body = CreateMusicRequest {
        data: RequestParams {
            r#type: "generations".to_string(),
            attributes: Attributes {
                prompts: vec![Prompt {
                    text: prompt,
                    weight,
                }],
                length_seconds,
                seed,
            },
        },
    };

    let generate_music_endpoint = env::var("STABLE_AUDIO_GENERATE_MUSIC_ENDPOINT")
        .expect("STABLE_AUDIO_GENERATE_MUSIC_ENDPOINT is not set");

    // POSTリクエスト送信
    let response = client
        .post(generate_music_endpoint)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    // ステータスコードを出力
    println!("Status: {}", response.status());

    // レスポンスボディを取得
    let body = response.text().await?;
    println!("Response Body: {}", body);

    let result_url = serde_json::from_str::<CreateMusicResponse>(&body)
        .unwrap()
        .data[0]
        .clone()
        .links
        .result;
    Ok(result_url)
}

async fn download_file(url: String, auth_key: &str) -> Result<(), reqwest::Error> {
    println!("download url: {}", url);
    let file = File::create("audio.mp3").expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    let client = Client::new();

    // 必要に応じてヘッダーを作成
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(auth_key).unwrap());
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

    let mut attempt = 1;
    let response = loop {
        // GETリクエストを送信
        let response = client
            .get(url.clone())
            .headers(headers.clone())
            .send()
            .await?;

        println!("Status: {}", response.status());

        if !response.status().is_success() {
            println!("Failed to download file: {}", response.status());
            break None;
        }
        match response.status() {
            StatusCode::OK => break Some(response),
            StatusCode::ACCEPTED => {
                if attempt > 20 {
                    println!("Max attempts reached. Giving up.");
                    break None;
                } else {
                    let wait_sec = 3;
                    println!(
                        "{}",
                        format!("Accepted. retry {}, wait {}sec", attempt, wait_sec)
                    );
                    sleep(Duration::from_secs(wait_sec)).await;
                    attempt += 1;
                }
            }
            _ => {
                println!("Failed to download: {}", response.status());
                break None;
            }
        }
    };
    let Some(response) = response else {
        return Ok(());
    };

    let mut stream = response.bytes_stream();
    // ストリームデータをファイルに書き込む
    while let Ok(Some(chunk)) = stream.try_next().await {
        // let chunk = chunk.expect("cannot chunk");
        writer.write_all(&chunk).expect("cannot write");
    }
    println!("Audio data has been written to 'audio.mp3'");
    Ok(())
}
