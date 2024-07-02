use youtube_dl::YoutubeDl;

pub async fn download_clip(url: &str) {
    YoutubeDl::new(url)
        .download_to_async("/tmp/")
        .await
        .expect("Failed to download video");
}
