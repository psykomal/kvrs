# kvrs


## Features 

- [x] Persistent Key Value Store based on [Bitcask](https://riak.com/assets/bitcask-intro.pdf) Architecture
- [x] Basic [Size-tiered Compaction](https://opensource.docs.scylladb.com/stable/kb/compaction.html#size-tiered-compaction-strategy-stcs)
- [x] Server and CLI Client 
- [x] Server with threadpool using Custom Thread Pool and Rayon
- [x] Raft Integration (using little-raft) + Axum server
- [ ] Benchmarks for every engine
- [x] Benchmarks for Raft Integration using locust


## Future Scope

- [ ] Enhance docs with benchmarks and steps to run
- [ ] Implement Lock-free KV Store
- [ ] Multi-raft Support
- [ ] Bring in Async Rust for engine
- [ ] Convert little-raft to be async

### Testing Raft 

1. Run the cmds in different terminals
```
cargo run --bin raft-server -- --id 1 --addr 127.0.0.1:4001
cargo run --bin raft-server -- --id 2 --addr 127.0.0.1:4002
cargo run --bin raft-server -- --id 3 --addr 127.0.0.1:4003
```

2. Wait for them to establish a leader

3. Fire REST API calls to the leader

Set Key :
```curl
GET /set?key=k2&val=v12 HTTP/1.1
Host: 127.0.0.1:4001
Connection: close
User-Agent: RapidAPI/4.2.0 (Macintosh; OS X/14.1.0) GCDHTTPRequest
```

Get Key :
```curl
GET /get?key=k2 HTTP/1.1
Host: 127.0.0.1:4003
Connection: close
User-Agent: RapidAPI/4.2.0 (Macintosh; OS X/14.1.0) GCDHTTPRequest
```
