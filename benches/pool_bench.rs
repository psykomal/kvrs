use assert_cmd::prelude::*;
use crossbeam_channel::Receiver;
use predicates::str::contains;
use std::process::Command;
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Barrier};
use std::time::Duration;
use std::{fmt::format, path::PathBuf, thread};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kvs::{KvStore, KvsEngine, SharedQueueThreadPool, SledKvsEngine, ThreadPool};
use rand::prelude::*;
use tempfile::TempDir;

fn write_queued_kvstore(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_queued_kvstore");
    for i in &vec![1, 2, 4, 8] {
        group.bench_with_input(format!("kvs_{}", i), i, |b, i| {
            let (sender, receiver) = mpsc::sync_channel::<()>(0);
            let addr = "127.0.0.1:4004";
            let temp_dir = TempDir::new().unwrap();
            let mut server = Command::cargo_bin("kvs-server").unwrap();
            let mut child = server
                .args(&[
                    "--engine",
                    "kvs",
                    "--addr",
                    addr,
                    "--threads",
                    i.to_string().as_str(),
                ])
                .current_dir(&temp_dir)
                .spawn()
                .unwrap();
            let handle = thread::spawn(move || {
                let _ = receiver.recv(); // wait for main thread to finish
                child.kill().expect("server exited before killed");
            });
            thread::sleep(Duration::from_secs(1));

            b.iter(|| {
                let barrier = Arc::new(Barrier::new(100));
                for j in 1..100 {
                    let barrier = barrier.clone();
                    thread::spawn(move || {
                        let key = format!("key{}", j);
                        let val = format!("val{}", j);
                        Command::cargo_bin("kvs-client")
                            .unwrap()
                            .args(&["set", key.as_str(), val.as_str(), "--addr", addr])
                            .assert()
                            .success();
                        barrier.wait();
                    });
                }
                barrier.wait();
            });

            // check values
            for j in 1..100 {
                let key = format!("key{}", j);
                let val = format!("val{}", j);
                Command::cargo_bin("kvs-client")
                    .unwrap()
                    .args(&["get", key.as_str(), "--addr", addr])
                    .assert()
                    .success()
                    .stdout(contains(val));
            }
        });
    }
}

criterion_group!(benches, write_queued_kvstore);
criterion_main!(benches);
