use futures::stream::StreamExt;
use snafu::ResultExt;
use tokio_quiche::http3::settings::Http3Settings;
use tokio_quiche::listen;
use tokio_quiche::metrics::DefaultMetrics;
use tokio_quiche::ConnectionParams;
use tokio_quiche::ServerH3Driver;

#[tokio::main]
async fn main() -> Result<(), snafu::Whatever> {

    let socket = tokio::net::UdpSocket::bind("localhost:1443").await.whatever_context("creating socket failed")?;
    let mut listeners =
        listen([socket], ConnectionParams::default(), DefaultMetrics).whatever_context("listening on socket failed")?;
    let accept_stream = &mut listeners[0];

    // Build a client
    let client = wreq::Client::builder()
        .emulation(wreq_util::Emulation::Safari26)
        .build().whatever_context("building wreq client failed")?;

    // Use the API you're already familiar with
    let resp = client.get("https://tls.peet.ws/api/all").send().await.whatever_context("wreq request failed")?;
    println!("{}", resp.text().await.whatever_context("wreq response failed")?);

    while let Some(conn) = accept_stream.next().await {
        let (driver, mut controller) =
            ServerH3Driver::new(Http3Settings::default());
        conn.whatever_context("accepting on connection failed")?.start(driver);

        tokio::spawn(async move {
            // `controller` is the handle to our established HTTP/3 connection.
            // For example, inbound requests are available as H3Events via:
            let _event = controller.event_receiver_mut().recv().await;
        });
    }

    Ok(())
}
