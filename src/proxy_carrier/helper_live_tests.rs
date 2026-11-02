//! Quill's live host against the real `quill-webview --carrier` and a
//! local echo page. Needs a desktop session and a debug helper build:
//!
//! `cargo build -p quill-webview && QUILL_CARRIER_HELPER=target/debug/quill-webview
//!  QUILL_WEBVIEW_CARRIER_LOOPBACK=1 cargo test --no-default-features --lib
//!  helper_live -- --ignored`
use super::helper::{Capability, HelperCarrierHost, HelperConfig};
use super::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const ECHO_PAGE: &str = r#"<!doctype html><html><head><script>
const b = window.TelegramWebProxy;
b.onmessage = (e) => b.postMessage(new Uint8Array(e.data).slice().reverse());
b.postMessage('{"t":"hello"}');
</script></head><body></body></html>"#;

fn serve() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
                    break;
                }
            }
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{ECHO_PAGE}",
                ECHO_PAGE.len()
            );
            let mut sink = Vec::new();
            let _ = stream.read_to_end(&mut sink);
        }
    });
    port
}

/// A relay that hands each request to the test and pokes
/// [`crate::wake::UPDATES`], the signal a carrier failure pokes too.
fn relay() -> (RelaySink, Receiver<serde_json::Value>) {
    let (tx, rx) = channel();
    let tx = Mutex::new(tx);
    let sink: RelaySink = Arc::new(move |request: &str| {
        let value = serde_json::from_str(request).unwrap();
        let sent = tx.lock().unwrap().send(value).is_ok();
        crate::wake::UPDATES.poke();
        sent
    });
    (sink, rx)
}

/// Keep-alives (one per helper tick, 3 s) a relay may take before the
/// test gives up on the page.
const KEEP_ALIVES_PER_RELAY: u64 = 5;

/// The next relayed message as `(carrier_id, message)`. A helper that
/// dies or goes quiet instead fails the test through `poll`: its watchdog
/// reports it and pokes the same signal. A helper that stays alive while
/// its page never relays fails it after [`KEEP_ALIVES_PER_RELAY`]
/// keep-alives.
fn next(
    host: &mut HelperCarrierHost,
    relayed: &Receiver<serde_json::Value>,
) -> (i64, serde_json::Value) {
    let request = wait_for_relay(relayed, &super::helper::KEEP_ALIVES_HEARD, || {
        let failures = host.poll();
        assert!(failures.is_empty(), "carrier failed: {failures:?}");
    });
    assert_eq!(request["@type"], "processWebProxyCarrierMessage");
    assert_eq!(request["@extra"], CARRIER_RELAY_EXTRA.as_extra());
    (
        request["carrier_id"].as_i64().unwrap(),
        request["message"].clone(),
    )
}

/// The next request on `relayed`, checking `poll` between wake-ups, and
/// failing once more than [`KEEP_ALIVES_PER_RELAY`] keep-alives counted
/// in `alive` came with none.
fn wait_for_relay(
    relayed: &Receiver<serde_json::Value>,
    alive: &std::sync::atomic::AtomicU64,
    mut poll: impl FnMut(),
) -> serde_json::Value {
    let first = alive.load(std::sync::atomic::Ordering::Acquire);
    loop {
        let seen = crate::wake::UPDATES.epoch();
        match relayed.try_recv() {
            Ok(request) => return request,
            Err(TryRecvError::Disconnected) => panic!("the relay was dropped"),
            Err(TryRecvError::Empty) => {}
        }
        poll();
        let alives = alive
            .load(std::sync::atomic::Ordering::Acquire)
            .saturating_sub(first);
        assert!(
            alives <= KEEP_ALIVES_PER_RELAY,
            "the helper stayed alive for {alives} ticks and the page relayed nothing"
        );
        futures_lite::future::block_on(crate::wake::UPDATES.wait_since(seen));
    }
}

#[test]
fn a_page_that_never_relays_fails_after_some_keep_alives() {
    let alive = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (_keep, relayed) = channel::<serde_json::Value>();
    let helper = {
        let (alive, stop) = (alive.clone(), stop.clone());
        std::thread::spawn(move || {
            while !stop.load(std::sync::atomic::Ordering::Acquire) {
                alive.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                crate::wake::UPDATES.poke();
                std::thread::yield_now();
            }
        })
    };
    let waited = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        wait_for_relay(&relayed, &alive, || {})
    }));
    stop.store(true, std::sync::atomic::Ordering::Release);
    helper.join().unwrap();
    let message = waited.unwrap_err();
    let message = message
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default();
    assert!(message.contains("relayed nothing"), "{message}");
}

#[test]
#[ignore = "needs a desktop session and QUILL_CARRIER_HELPER"]
fn helper_live_relay_round_trip() {
    let program = std::env::var_os("QUILL_CARRIER_HELPER").expect("QUILL_CARRIER_HELPER");
    let port = serve();
    let origin = format!("http://127.0.0.1:{port}");
    let data_dir = crate::scratch::process_dir("helper-live").join("carrier");
    let (sink, relayed) = relay();
    let mut host = HelperCarrierHost::new(HelperConfig {
        program: Some(program.into()),
        data_dir: Some(data_dir.clone()),
        probe_cache: None,
        relay: sink,
        capability: Capability::default(),
    });
    let request = CarrierRequest {
        id: 21,
        proxy_id: 1,
        url: CarrierUrl::new(format!("{origin}/app/?bridge=cap#android=n")),
        origin,
        path_prefix: "/app/".into(),
        bridge_object_name: "TelegramWebProxy".into(),
        temporary: false,
    };
    host.open(&request).unwrap();
    // Sent before the page listens: held until it does.
    host.deliver(21, "AQID".into());
    let (carrier, hello) = next(&mut host, &relayed);
    assert_eq!(carrier, 21);
    assert_eq!(hello["@type"], "webProxyCarrierMessageText");
    assert_eq!(hello["text"], r#"{"t":"hello"}"#);
    let (_, early) = next(&mut host, &relayed);
    assert_eq!(early["data"], "AwIB");
    // Relay latency through the pipe and the page, both ways.
    let mut rounds = Vec::new();
    for _ in 0..20 {
        let started = Instant::now();
        host.deliver(21, "AQID".into());
        let (carrier, message) = next(&mut host, &relayed);
        assert_eq!(carrier, 21);
        assert_eq!(message["@type"], "webProxyCarrierMessageBinary");
        assert_eq!(message["data"], "AwIB");
        rounds.push(started.elapsed());
    }
    rounds.sort();
    eprintln!(
        "round trip median {:?}, max {:?}",
        rounds[rounds.len() / 2],
        rounds[rounds.len() - 1]
    );
    assert!(host.poll().is_empty());
    host.close(21, true);
    host.shutdown();
    assert!(host.poll().is_empty());
    let left: Vec<_> = std::fs::read_dir(&data_dir)
        .map(|entries| entries.flatten().map(|entry| entry.file_name()).collect())
        .unwrap_or_default();
    assert!(left.is_empty(), "{left:?}");
}

#[test]
#[ignore = "needs a desktop session and QUILL_CARRIER_HELPER"]
fn helper_live_capability_check_passes() {
    let program = std::env::var_os("QUILL_CARRIER_HELPER").expect("QUILL_CARRIER_HELPER");
    let data_dir = crate::scratch::process_dir("helper-live").join("probe");
    let started = Instant::now();
    assert_eq!(
        super::helper::probe(std::path::Path::new(&program), &data_dir),
        Some(true)
    );
    eprintln!("capability check took {:?}", started.elapsed());
}
