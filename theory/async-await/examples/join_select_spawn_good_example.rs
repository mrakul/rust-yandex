use tokio::time::{sleep, Duration};

async fn task(name: &str, delay_ms: u64) {
    println!("🚀 {} started", name);
    sleep(Duration::from_millis(delay_ms)).await;
    println!("✅ {} finished", name);
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    println!("=== Sequential .await ===");
    task("A", 300).await;
    task("B", 300).await;
    
    println!("\n=== join! (concurrent, wait all) ===");
    tokio::join!(task("C", 300), task("D", 300));
    
    println!("\n=== select! (concurrent, wait first) ===");
    tokio::select! {
        _ = task("Fast", 100) => println!("Fast won!"),
        _ = task("Slow", 1000) => println!("Slow won!"),
    }
    
    println!("\n=== spawn (background) ===");
    let spawned_task = tokio::spawn(task("Background", 500));
    println!("Main continues immediately!");
    spawned_task.await.ok();
}