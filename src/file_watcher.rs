
use notify::RecommendedWatcher;
use tokio::sync::mpsc::Sender;


// Receive a list of paths to watch and a channel to send the events to
pub async fn create_watcher(tx: Sender<()>) -> notify::Result<(RecommendedWatcher)> {
  
  let watcher = notify::recommended_watcher( move|res| {
    let tx = tx.clone();
    tokio::task::spawn(async move {
      match res {
        Ok(event) => {
          println!("event: {:?}", event);
          tx.send(()).await.unwrap();
        },
        Err(e) => println!("watch error: {:?}", e),
      }
    });
  })?;

  Ok(watcher)
}
