use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;

const USER_AGENT: &str = "com.google.android.youtube/17.36.4 (Linux; U; Android 12; GB) gzip";

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    #[serde(rename = "videoDetails")]
    pub video_details: VideoDetails,
    #[serde(rename = "streamingData")]
    pub streaming_data: StreamingData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamingData {
    pub formats: Option<Vec<StreamingFormat>>,
    #[serde(rename = "adaptiveFormats")]
    pub adaptive_formats: Option<Vec<StreamingFormat>>,
    #[serde(rename = "dashManifestUrl")]
    pub dash_manifest_url: Option<String>,
    #[serde(rename = "hlsManifestUrl")]
    pub hls_manifest_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamingFormat {
    pub url: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub bitrate: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: String,
    pub fps: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoDetails {
    #[serde(rename = "videoId")]
    pub video_id: String,
    pub title: String,
}

pub async fn get_video_info(id: &str) -> VideoInfo {
    // https://tyrrrz.me/blog/reverse-engineering-youtube-revisited
    // https://github.com/Tyrrrz/YoutubeExplode

    let client = reqwest::Client::new();
    let req = client
        .post("https://www.youtube.com/youtubei/v1/player")
        .header("User-Agent", USER_AGENT)
        .json(&json!({
            "videoId": id,
            "context": {
                "client": {
                    "clientName": "ANDROID_TESTSUITE",
                    "clientVersion": "1.9",
                    "androidSdkVersion": 30,
                    "hl": "en",
                    "gl": "US",
                    "utcOffsetMinutes": 0
                }
            }
        }));

    let resp = req.send().await.unwrap();
    let text = resp.text().await.unwrap();

    serde_json::from_str::<VideoInfo>(&text).unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoListResponse {
    pub etag: String,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: String,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub items: Vec<VideoListItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoListItem {
    pub etag: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "totalResults")]
    total_results: i32,
    #[serde(rename = "resultsPerPage")]
    results_per_page: i32,
}

// GET https://youtube.googleapis.com/youtube/v3/videos?part=snippet%2CcontentDetails%2Cstatistics&chart=mostPopular&regionCode=US&key=[YOUR_API_KEY] HTTP/1.1
// Authorization: Bearer [YOUR_ACCESS_TOKEN]
// Accept: application/json

pub async fn video_chart(region_code: &str, api_key: &str) -> VideoListResponse {
    let client = reqwest::Client::new();
    let req = client
        .get("https://youtube.googleapis.com/youtube/v3/videos")
        .header("User-Agent", USER_AGENT)
        .header("Accept","application/json")
        .query(&[
            ("regionCode", region_code),
            ("chart", "mostPopular"),
            ("part", "snippet"),
            ("key", api_key),
        ]);

    let resp = req.send().await.unwrap();
    let text = resp.text().await.unwrap();

    serde_json::from_str::<VideoListResponse>(&text).unwrap()
}

fn rank_format(fmt: StreamingFormat) -> (i32, i32) {
    (
        if fmt.mime_type.contains("avc1") {
            10
        } else {
            0
        },
        match fmt.quality.as_str() {
            "1080p" => 0,
            "hd720" => 8,
            "medium" => 4,
            "small" => 2,
            _ => 1,
        },
    )
}

pub fn find_best_format(formats: Vec<StreamingFormat>) -> Option<StreamingFormat> {
    let mut best: Option<StreamingFormat> = None;
    let mut best_quality = 0;
    let mut best_codec = 0;

    for curr in formats {
        if let Some(_) = best {
            let (curr_codec, curr_quality) = rank_format(curr.clone());

            if curr_codec > best_codec || (curr_codec == best_codec && curr_quality > best_quality)
            {
                best = Some(curr.clone());
            }
        } else {
            (best_codec, best_quality) = rank_format(curr.clone());
            best = Some(curr.clone());
        }
    }

    best
}
