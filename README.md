[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/ehttpd-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/ehttpd-rust)
[![docs.rs](https://docs.rs/ehttpd/badge.svg)](https://docs.rs/ehttpd)
[![crates.io](https://img.shields.io/crates/v/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![Download numbers](https://img.shields.io/crates/d/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![dependency status](https://deps.rs/crate/ehttpd/latest/status.svg)](https://deps.rs/crate/ehttpd)


# `ehttpd`
Welcome to `ehttpd` 🎉

`ehttpd` is a thread-based HTTP server library, which can be used to create custom HTTP server applications.


## Thread-based design
The rationale behind the thread-based approach is that it is much easier to implement than `async/await`, subsequently
requires less code, and is – in theory – less error prone.

Furthermore, it also simplifies application development since the developer cannot accidentally stall the entire runtime
with a single blocking call – managed by the OS-scheduler, threads offer much stronger concurrency isolation guarantees
(which can even be `nice`d or tweaked in most environments if desired).


## Performance
While the thread-based approach is not the most efficient out there, it's not that bad either. Some `wrk` benchmarks:

### MacBook Pro (`M1 Pro`, `helloworld`, `v0.7.0`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.76ms    3.05ms  46.96ms   91.85%
    Req/Sec     1.00k   565.47     4.64k    79.33%
  640123 requests in 10.09s, 31.75MB read
Requests/sec:  63468.72
Transfer/sec:      3.15MB
```

### Old Linux Machine (`Intel(R) Core(TM) i5-2500K CPU @ 3.30GHz`, `helloworld-nokeepalive`, `v0.7.0`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.22ms    1.00ms  60.93ms   95.30%
    Req/Sec   435.19     56.94     1.00k    85.05%
  278046 requests in 10.10s, 18.83MB read
Requests/sec:  27528.42
Transfer/sec:      1.86MB
```
