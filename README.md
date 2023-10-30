# kvrs


## Features / Work Donw

- [x] Persistent Key Value Store based on [Bitcask](https://riak.com/assets/bitcask-intro.pdf) Architecture
- [x] Basic [Size-tiered Compaction](https://opensource.docs.scylladb.com/stable/kb/compaction.html#size-tiered-compaction-strategy-stcs)
- [x] Server and CLI Client 
- [x] Server with threadpool using Custom Thread Pool and Rayon
- [x] Raft Integration + Actix-web server
- [x] Benchmarks with Sled Engine (PS: Sled absolutely blows this out of the water)
- Benchmarks for Raft Integration using locust


## Future Scope

- [ ] Implement Lock-free KV Store
- [ ] Bring in Async Rust
- [ ] Multi-raft Support