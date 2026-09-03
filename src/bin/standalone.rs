//! Noob CompressorLab without a DAW: a fake audio thread runs a demo source
//! (vocal, bass, drums, noise, tones) through the active model at 48 kHz /
//! 256 samples, publishes the meter, cell and transfer streams, and serves
//! the SPA from `web/dist` (or lets `vite` proxy to it).
//!
//! ```text
//! cargo run --bin noob-compressorlab-standalone -- [--port N] [--open] [--dir path]
//! ```
//!
//! | flag | meaning |
//! |---|---|
//! | `--port N` | insist on port `N` (otherwise 4244, walking up if taken) |
//! | `--open` | open the page in the system browser |
//! | `--dir path` | serve this directory instead of `web/dist` |
//!
//! The page's own state (presets, window size) persists in a file through
//! noob-vst-webgui-framework's `FileStore`; the plug-in keeps the same data inside
//! its host state. A `status` message goes out once a second with the
//! client count, block count, edit count and the active model's latency.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use noob_compressorlab::dsp::{self, Processor, Source};
use noob_vst_webgui_framework::{AudioHandle, FileStore, ServerConfig};
use serde_json::json;

const SR: f32 = 48_000.0;
const BLOCK: usize = 256;

struct Stats {
    blocks: AtomicU64,
    latency: AtomicUsize,
}

/// The fake audio thread: generate, compress, publish, sleep until the next block.
fn audio_thread(mut audio: AudioHandle, ix: dsp::ParamIx, stats: Arc<Stats>) {
    let mut processor = Processor::new(SR);
    let mut source = Source::new(0x9E37_79B9);
    let mut l = vec![0.0f32; BLOCK];
    let mut r = vec![0.0f32; BLOCK];
    let block_dur = Duration::from_secs_f64(BLOCK as f64 / SR as f64);
    let mut next = Instant::now();
    let mut n: u64 = 0;
    loop {
        let settings = dsp::read_settings(&audio, &ix);
        processor.configure(&settings);
        let kind = ix
            .src_kind
            .map(|i| audio.param(i).round() as usize)
            .unwrap_or(0);
        let freq = ix.src_freq.map(|i| audio.param(i)).unwrap_or(110.0);
        let level = ix.src_level.map(|i| audio.param(i)).unwrap_or(0.4);
        for i in 0..BLOCK {
            let x = source.next(kind, freq, SR) * level;
            l[i] = x;
            r[i] = x * 0.95;
        }
        processor.process(&mut l, &mut r);
        processor.publish(&mut audio);
        n += 1;
        stats.blocks.store(n, Ordering::Relaxed);
        stats.latency.store(processor.latency(), Ordering::Relaxed);

        next += block_dur;
        let now = Instant::now();
        if next > now {
            thread::sleep(next - now);
        } else if now - next > Duration::from_millis(200) {
            next = now;
        }
    }
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let r = std::process::Command::new("xdg-open").arg(url).spawn();
    if let Err(e) = r {
        log::warn!("could not open browser: {e}");
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut port: Option<u16> = None;
    let mut open = false;
    let mut dir: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" | "-p" => port = args.next().and_then(|v| v.parse().ok()),
            "--open" | "-o" => open = true,
            "--dir" | "-d" => dir = args.next().map(PathBuf::from),
            "-h" | "--help" => {
                println!("noob-compressorlab standalone [--port N] [--open] [--dir path]");
                return;
            }
            other => log::warn!("ignoring argument {other}"),
        }
    }
    let web = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/web"));
    let dist = web.join("dist");
    let (dir, built) = match dir {
        Some(d) => (d, true),
        None => (dist.clone(), dist.join("index.html").is_file()),
    };
    let dir = dir.canonicalize().unwrap_or(dir);

    let (bridge, ix) = dsp::build_bridge("noob-compressorlab", SR);
    let audio = bridge.take_audio().expect("audio handle");
    let stats = Arc::new(Stats {
        blocks: AtomicU64::new(0),
        latency: AtomicUsize::new(0),
    });
    {
        let stats = stats.clone();
        thread::Builder::new()
            .name("fake-audio".into())
            .spawn(move || audio_thread(audio, ix, stats))
            .expect("spawn audio thread");
    }

    let store = FileStore::attach(&bridge, FileStore::default_path("noob-compressorlab"));
    let cfg = match port {
        Some(p) => ServerConfig::default().port(p),
        None => ServerConfig::default().prefer_port(4244),
    };
    let server =
        noob_vst_webgui_framework::serve(&bridge, cfg.assets_dir(&dir)).expect("start server");
    println!();
    println!("  noob-compressorlab standalone {}", server.url());
    println!("  websocket                     {}", server.ws_url());
    println!("  assets                        {}", dir.display());
    println!("  ui store                      {}", store.path().display());
    if !built {
        println!();
        println!("  web/dist not found. Either build the SPA once:");
        println!("      cd web && npm install && npm run build");
        println!("  or develop with hot reload (proxies /ws to this server):");
        println!(
            "      cd web && NOOB_VST_WEBGUI_FRAMEWORK_PORT={} npm run dev",
            server.port()
        );
    }
    println!();
    if open {
        open_browser(&server.url());
    }

    let mut last_status = Instant::now();
    let mut edits = 0u64;
    loop {
        bridge.drain_edits(|_| edits += 1);
        while let Some(m) = bridge.poll_message() {
            match m.topic.as_str() {
                "reset" => {
                    for i in 0..bridge.param_count() {
                        let d = bridge.spec(i).map(|s| s.default).unwrap_or(0.0);
                        bridge.set_param(i, d);
                    }
                }
                "resize" | "fullscreen" => {}
                other => log::info!("message from client {}: {other} {}", m.client, m.data),
            }
        }
        if last_status.elapsed() >= Duration::from_secs(1) {
            last_status = Instant::now();
            let latency = stats.latency.load(Ordering::Relaxed);
            bridge.send_json(
                "status",
                json!({
                    "clients": server.client_count(),
                    "blocks": stats.blocks.load(Ordering::Relaxed),
                    "edits": edits,
                    "dropped": bridge.dropped_ui_changes(),
                    "sample_rate": SR,
                    "block": BLOCK,
                    "latency_samples": latency,
                    "latency_ms": latency as f32 * 1000.0 / SR,
                }),
            );
        }
        if let Err(e) = store.flush() {
            log::warn!("could not save the UI store: {e}");
        }
        thread::sleep(Duration::from_millis(5));
    }
}
