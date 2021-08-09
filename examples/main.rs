use udp_logger_rs::info;
fn main() {
    femme::start();

    // The standard logging we know and love
    info!("hello");
    info!("hello",);
    info!("hello {}", "cats");
    info!("hello {}", "cats",);
    info!(target: "MyApp", "hello");
    info!(target: "MyApp", "hello",);
    info!(target: "MyApp", "hello {}", "cats");
    info!(target: "MyApp", "hello {}", "cats",);

    // The kv logging we hope to love
    let ctx: Vec<(String, String)> = vec![
        ("key1".into(), "value1".into()),
        ("key2".into(), "value2".into()),
    ];

    // default target
    info!(kvs: &ctx, "hello");
    info!(kvs: &ctx, "hello",);
    info!(kvs: &ctx, "hello {}", "cats");
    info!(kvs: &ctx, "hello {}", "cats",);

    // with target provided
    info!(target: "MyApp", kvs: &ctx, "hello");
    info!(target: "MyApp", kvs: &ctx, "hello",);
    info!(target: "MyApp", kvs: &ctx, "hello {}", "cats");
    info!(target: "MyApp", kvs: &ctx, "hello {}", "cats",);
}
