#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let mut client = sonar_grpc::client("http://10.0.0.190:3000").await?;
    let tracks = sonar_grpc::ext::track_list_all(&mut client).await?;
    println!("{:?}", tracks);

    return Ok(());

    let (player, _events) = sonar_player::AudioPlayer::new();
    // let file = tokio::fs::File::open("test.mp3").await?;
    // let source = sonar_player::FileAudioSource::new(file).await?;

    let client =
        sonar_player::AudioClientGrpc::new(sonar_grpc::client("http://10.0.0.190:3000").await?);
    let source =
        sonar_player::AudioStream::new(client, "sonar:track:3000005".parse().unwrap()).await?;

    player.source(source);
    player.play();

    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    Ok(())
}
