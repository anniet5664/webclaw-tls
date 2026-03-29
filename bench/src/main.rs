use std::time::Instant;
use webclaw_http::Client;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .chrome()
        .build()
        .expect("build");

    let sites = vec![
        ("HN", "https://news.ycombinator.com"),
        ("Wikipedia", "https://en.wikipedia.org/wiki/Web_scraping"),
        ("GitHub", "https://github.com/0xMassi/webclaw"),
        ("Stripe", "https://stripe.com"),
        ("Cloudflare", "https://www.cloudflare.com/learning/what-is-cloudflare/"),
        ("Nike", "https://www.nike.com/w/mens-shoes-nik1zy7ok"),
        ("httpbin", "https://httpbin.org/get"),
    ];

    // Warm up: one request to establish that the client works
    client.get("https://httpbin.org/get").await.ok();

    println!("=== PROFILING: Cold vs Warm requests ===");
    println!("{:20} {:>8} {:>8} {:>8} {:>8}", "Site", "Cold", "Warm1", "Warm2", "Warm3");
    println!("{}", "-".repeat(60));

    let mut cold_total = 0u128;
    let mut warm_total = 0u128;
    let mut count = 0;

    for (name, url) in &sites {
        // Cold (new connection)
        let start = Instant::now();
        client.get(url).await.ok();
        let cold = start.elapsed().as_millis();

        // Warm (reused connection via HTTP/2)
        let mut warms = Vec::new();
        for _ in 0..3 {
            let start = Instant::now();
            client.get(url).await.ok();
            warms.push(start.elapsed().as_millis());
        }

        cold_total += cold;
        warm_total += warms.iter().sum::<u128>();
        count += 1;

        println!(
            "{:20} {:>7}ms {:>7}ms {:>7}ms {:>7}ms",
            name, cold, warms[0], warms[1], warms[2]
        );
    }

    println!("{}", "-".repeat(60));
    println!(
        "{:20} {:>7}ms {:>7}ms",
        "AVERAGE",
        cold_total / count as u128,
        warm_total / (count * 3) as u128,
    );

    // Sequential same-host burst (tests HTTP/2 multiplexing)
    println!("\n=== HTTP/2 CONNECTION REUSE (20 requests to httpbin) ===");
    let start = Instant::now();
    for _ in 0..20 {
        client.get("https://httpbin.org/get").await.ok();
    }
    let total = start.elapsed();
    let avg = total.as_millis() / 20;
    println!("  Total: {}ms, Avg: {}ms/req, Throughput: {:.1} req/s",
        total.as_millis(), avg, 20.0 / total.as_secs_f64());

    // Concurrent same-host burst
    println!("\n=== CONCURRENT BURST (20 parallel to httpbin) ===");
    let start = Instant::now();
    let mut handles = Vec::new();
    for i in 0..20 {
        let c = client.clone();
        handles.push(tokio::spawn(async move {
            let url = format!("https://httpbin.org/get?i={i}");
            c.get(&url).await.map(|r| r.status())
        }));
    }
    let mut ok = 0;
    for h in handles {
        if let Ok(Ok(200)) = h.await { ok += 1; }
    }
    let total = start.elapsed();
    println!("  {ok}/20 OK in {}ms ({:.1} req/s)",
        total.as_millis(), 20.0 / total.as_secs_f64());

    // Minimal request overhead test (measure just our code, not network)
    println!("\n=== OVERHEAD TEST (localhost would be ideal, using fastest remote) ===");
    let mut times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        client.get("https://httpbin.org/get").await.ok();
        times.push(start.elapsed().as_millis());
    }
    times.sort();
    println!("  Min: {}ms, Median: {}ms, Max: {}ms",
        times[0], times[5], times[9]);
}
