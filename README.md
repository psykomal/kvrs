# kvrs


## Features 

- [x] Persistent Key Value Store based on [Bitcask](https://riak.com/assets/bitcask-intro.pdf) Architecture
- [x] Basic [Size-tiered Compaction](https://opensource.docs.scylladb.com/stable/kb/compaction.html#size-tiered-compaction-strategy-stcs)
- [x] Server and CLI Client 
- [x] Server with threadpool using Custom Thread Pool and Rayon
- [x] Raft Integration (using openraft) + Actix-web server
- [x] Benchmarks with Sled Engine (PS: Sled absolutely blows the custom KVstore out of the water)
- [ ] Integrate with raft-rs instead of openraft (openraft seems to be breaking at high load)
- [ ] Benchmarks for Raft Integration using locust


## Future Scope

- [ ] Enhance docs with benchmarks and steps to run
- [ ] Implement Lock-free KV Store
- [ ] Bring in Async Rust
- [ ] Multi-raft Support

### Testing Raft 

 Follow the steps here - https://github.com/datafuselabs/openraft/tree/main/examples/raft-kv-memstore


