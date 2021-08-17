use lazy_static::lazy_static;
use udp_logger_rs::{debug, error, info, log, trace, warn, Level};
// A logger, which proxies to other loggers. This allows for each test to install
// a proxy.
#[derive(Default)]
struct ProxyLogger {
    logger: std::sync::Mutex<Option<udp_logger_rs::UdpLogger>>,
}

impl ProxyLogger {
    fn set_logger(&self, logger: udp_logger_rs::UdpLogger) {
        let logger = logger.partial_init();
        *self.logger.lock().unwrap() = Some(logger);
    }
    fn log_interface(&self) -> &dyn log::Log {
        self
    }
}

impl log::Log for ProxyLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let guard = self.logger.lock().unwrap();
        if let Some(logger) = &*guard {
            return logger.enabled(metadata);
        }
        return false;
    }
    fn log(&self, record: &log::Record<'_>) {
        let guard = self.logger.lock().unwrap();
        if let Some(logger) = &*guard {
            logger.log(record);
        }
    }
    fn flush(&self) {}
}

lazy_static! {
    static ref PROXY_LOGGER: ProxyLogger = ProxyLogger::default();
}

#[derive(Default)]
struct TestLogContext {
    logctx: Vec<(String, String)>,
}

#[derive(Default)]
struct UdpClient {}
impl UdpClient {
    fn pkt_eq(socket: &std::net::UdpSocket, source: &str, ctx: &str) -> bool {
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut buf = [0; 4096];
        let (byte_count, src_addr) = socket.recv_from(&mut buf).expect("udp datagram");
        let src_addr = format!("{}", src_addr);
        assert_eq!(source, src_addr);
        assert!(byte_count > 0);
        let filled_buf = &mut buf[..byte_count];
        let str = std::str::from_utf8(&filled_buf).unwrap();
        let (time, contents) = str.split_at(23);
        //println!("time={} contents={}", time, contents);
        // 2021-08-09 18:41:50.336 INFO  [test] logging Info w/ kv as HashMap key1=Value1 Key2=Value2
        let parse_result = chrono::NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S%.3f");
        let cmp_result = ctx == contents;
        if !cmp_result {
            println!("mismatch got={}", contents);
        }
        return parse_result.is_ok() && cmp_result;
    }
}

//
// This is a basic test which confirms default logging is going out properly.
#[test]
fn test_macro() {
    let _result = log::set_logger(PROXY_LOGGER.log_interface());
    let udp_logger = udp_logger_rs::UdpLogger::default();
    PROXY_LOGGER.set_logger(udp_logger);

    let socket = std::net::UdpSocket::bind("127.0.0.1:4010").expect("unable to bind");
    socket
        .set_nonblocking(true)
        .expect("unable to set nonblocking");

    let mut kvs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    kvs.insert("key1".into(), "Value1".into());
    kvs.insert("Key2".into(), "Value2".into());
    log!(kvs: &kvs, Level::Info, "logging Info w/ kv as HashMap");

    let mut buf = [0; 4096];
    std::thread::sleep(std::time::Duration::from_millis(20));
    let (byte_count, _src_addr) = socket.recv_from(&mut buf).expect("udp datagram");
    assert!(byte_count > 0);

    let filled_buf = &mut buf[..byte_count];
    let str = std::str::from_utf8(&filled_buf).unwrap();
    let (time, ctx) = str.split_at(23);
    // 2021-08-09 18:41:50.336 INFO  [test] logging Info w/ kv as HashMap key1=Value1 Key2=Value2
    let _dt = chrono::NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S%.3f").unwrap();
    let same = ctx == " INFO  [test] logging Info w/ kv as HashMap key1=Value1 Key2=Value2"
        || ctx == " INFO  [test] logging Info w/ kv as HashMap Key2=Value2 key1=Value1";
    if !same {
        println!("{}", ctx);
    }
    assert!(same);

    let kvs: std::vec::Vec<(String, String)> = vec![
        ("key1".into(), "Value1".into()),
        ("Key2".into(), "Value2".into()),
    ];
    log!(kvs: &kvs, Level::Info, "logging Info w/ kv as Vec");
    assert!(UdpClient::pkt_eq(
        &socket,
        "127.0.0.1:4000",
        " INFO  [test] logging Info w/ kv as Vec key1=Value1 Key2=Value2"
    ));
}

//
// This tests that if the default source and destination addresses are changed, the logging
// honors those changes.
#[test]
fn non_default_log() {
    let _result = log::set_logger(PROXY_LOGGER.log_interface());
    let udp_logger = udp_logger_rs::UdpLogger::default()
        .with_source("127.0.0.1:4040")
        .with_destination("127.0.0.1:4041");
    PROXY_LOGGER.set_logger(udp_logger);

    let socket = std::net::UdpSocket::bind("127.0.0.1:4041").expect("unable to bind");
    socket
        .set_nonblocking(true)
        .expect("unable to set nonblocking");

    let kvs: std::vec::Vec<(String, String)> = vec![
        ("key1".into(), "Value1".into()),
        ("Key2".into(), "Value2".into()),
    ];
    let mut test = TestLogContext::default();
    test.logctx = kvs;
    let test = test;

    log!(target: "MyApp", Level::Error, "logging Error w/ target");
    assert!(UdpClient::pkt_eq(
        &socket,
        "127.0.0.1:4040",
        " ERROR [MyApp] logging Error w/ target"
    ));

    log!(target: "MyApp", Level::Warn, "{}", "parameterized logging Warn w/ target");
    assert!(UdpClient::pkt_eq(
        &socket,
        "127.0.0.1:4040",
        " WARN  [MyApp] parameterized logging Warn w/ target"
    ));

    log!(kvs: &test.logctx, Level::Info, "logging Info w/ kv");
    assert!(UdpClient::pkt_eq(
        &socket,
        "127.0.0.1:4040",
        " INFO  [test] logging Info w/ kv key1=Value1 Key2=Value2"
    ));

    log!(kvs: &test.logctx, Level::Debug, "{}", "parameterized logging Debug w/ kv");
    assert!(UdpClient::pkt_eq(
        &socket,
        "127.0.0.1:4040",
        " DEBUG [test] parameterized logging Debug w/ kv key1=Value1 Key2=Value2"
    ));

    log!(target: "MyApp", kvs: &test.logctx, Level::Trace, "logging Trace w/ target and kv");
    assert!(UdpClient::pkt_eq(
        &socket,
        "127.0.0.1:4040",
        " TRACE [MyApp] logging Trace w/ target and kv key1=Value1 Key2=Value2"
    ));

    log!(Level::Error, "error logging");
    assert!(UdpClient::pkt_eq(
        &socket,
        "127.0.0.1:4040",
        " ERROR [test] error logging"
    ));
}

//
// This tests that setting a source_level address and destination_level address, depending upon
// the level, the correct source and destiation are used in sending.
#[test]
fn multi_socket() {
    let _result = log::set_logger(PROXY_LOGGER.log_interface());
    let udp_logger = udp_logger_rs::UdpLogger::default()
        .with_source("127.0.0.1:4060")
        .with_source_level("127.0.0.1:4070", udp_logger_rs::LevelFilter::Info)
        .with_destination("127.0.0.1:4061")
        .with_destination_level("127.0.0.1:4071", udp_logger_rs::LevelFilter::Info);
    PROXY_LOGGER.set_logger(udp_logger);

    let debug_trace_socket = std::net::UdpSocket::bind("127.0.0.1:4061").expect("unable to bind");
    debug_trace_socket
        .set_nonblocking(true)
        .expect("unable to set nonblocking");

    let error_warn_info_socket =
        std::net::UdpSocket::bind("127.0.0.1:4071").expect("unable to bind");
    error_warn_info_socket
        .set_nonblocking(true)
        .expect("unable to set nonblocking");

    let kvs: std::vec::Vec<(String, String)> = vec![
        ("key1".into(), "Value1".into()),
        ("Key2".into(), "Value2".into()),
    ];
    let mut test = TestLogContext::default();
    test.logctx = kvs;
    let test = test;

    trace!(kvs: &test.logctx, "trace logging w/ kv");
    assert!(UdpClient::pkt_eq(
        &debug_trace_socket,
        "127.0.0.1:4060",
        " TRACE [test] trace logging w/ kv key1=Value1 Key2=Value2"
    ));

    debug!(target: "MyApp", "debug logging w/ target");
    assert!(UdpClient::pkt_eq(
        &debug_trace_socket,
        "127.0.0.1:4060",
        " DEBUG [MyApp] debug logging w/ target"
    ));

    info!(kvs: &test.logctx, "info logging w/ kv");
    assert!(UdpClient::pkt_eq(
        &error_warn_info_socket,
        "127.0.0.1:4070",
        " INFO  [test] info logging w/ kv key1=Value1 Key2=Value2"
    ));

    warn!(kvs: &test.logctx, "{}", "warn parameterized logging w/ kv");
    assert!(UdpClient::pkt_eq(
        &error_warn_info_socket,
        "127.0.0.1:4070",
        " WARN  [test] warn parameterized logging w/ kv key1=Value1 Key2=Value2"
    ));

    error!(target: "MyApp", kvs: &test.logctx, "error logging w/ target and kv");
    assert!(UdpClient::pkt_eq(
        &error_warn_info_socket,
        "127.0.0.1:4070",
        " ERROR [MyApp] error logging w/ target and kv key1=Value1 Key2=Value2"
    ));
}
