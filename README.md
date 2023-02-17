[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/ehttpd-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/ehttpd-rust)
[![docs.rs](https://docs.rs/ehttpd/badge.svg)](https://docs.rs/ehttpd)
[![crates.io](https://img.shields.io/crates/v/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![Download numbers](https://img.shields.io/crates/d/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![dependency status](https://deps.rs/crate/ehttpd/0.4.1/status.svg)](https://deps.rs/crate/ehttpd/0.4.1)


# `ehttpd`
Welcome to `ehttpd` 🎉

`ehttpd` is a thread-based HTTP server library, which can be used to create custom HTTP server applications.


## Thread-based design
The rationale behind the thread-based approach is that it is much easier to implement than `async/await`, subsequently requires less codes, and is – in theory – less error prone.

Furthermore, it also simplifies application development since the developer cannot accidentally stall the entire runtime
with a single blocking call – via the OS' scheduler, threads offer much strong concurrency isolation guarantees, which
in most environments can even be `nice`d or tweaked if appropriate.


## Performance
While the thread-based approach is not the most efficient out there, it's not that bad either. Some `wrk` benchmarks:

### MacBook Pro (`M1 Pro`, `helloworld`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.98ms  346.32us  13.64ms   89.26%
    Req/Sec     1.03k   125.94     1.38k    70.02%
  662807 requests in 10.10s, 32.87MB read
Requests/sec:  65622.88
Transfer/sec:      3.25MB
```

### Old Linux Machine (`Intel(R) Core(TM) i5-2500K CPU @ 3.30GHz`, `helloworld-nokeepalive`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.31ms    1.31ms  79.98ms   97.48%
    Req/Sec   419.80     73.74     2.03k    94.15%
  268421 requests in 10.10s, 18.18MB read
Requests/sec:  26579.74
Transfer/sec:      1.80MB
```
