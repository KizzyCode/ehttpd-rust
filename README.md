[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/ehttpd-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/ehttpd-rust)
[![docs.rs](https://docs.rs/ehttpd/badge.svg)](https://docs.rs/ehttpd)
[![crates.io](https://img.shields.io/crates/v/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![Download numbers](https://img.shields.io/crates/d/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![dependency status](https://deps.rs/crate/ehttpd/latest/status.svg)](https://deps.rs/crate/ehttpd)


# `ehttpd`
Welcome to `ehttpd` 🎉

`ehttpd` is a HTTP server library, which can be used to create custom HTTP server applications. It also offers an
optional threadpool-based server for simple applications (feature: `server`, disabled by default).


## Threadpool-based server
The rationale behind the thread-based approach is that it is much easier to implement than `async/await`, subsequently
requires less code, and is – in theory – less error prone.

Furthermore, it also simplifies application development as the developer cannot accidentally stall the entire runtime
with a single blocking call. Since threads are managed and preempted by the OS-scheduler, they offer much stronger
concurrency guarantees, and are usually more resilient against optimization issues or bugs.


### Performance
While the thread-based approach is not the most efficient out there, it's not that bad either. Some `wrk` benchmarks:

#### MacBook Pro (`M1 Pro`, `v0.12.0`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     0.85ms   85.72us   6.62ms   92.48%
    Req/Sec     1.17k    28.34     1.25k    87.08%
  754610 requests in 10.10s, 37.42MB read
Requests/sec:  74713.82
Transfer/sec:      3.71MB

$ wrk -t 64 -c 64 http://localhost:9999/testolope-nokeepalive
Running 10s test @ http://localhost:9999/testolope-nokeepalive
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.96ms    4.81ms  89.48ms   98.57%
    Req/Sec   303.72     62.34   353.00     90.42%
  117823 requests in 10.06s, 7.98MB read
  Socket errors: connect 64, read 0, write 0, timeout 0
Requests/sec:  11716.92
Transfer/sec:    812.40KB
```

#### Linux Machine (`Intel(R) Core(TM) i5-10400F CPU @ 2.90GHz`, `v0.12.0`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   188.08us  177.57us  12.10ms   90.95%
    Req/Sec     5.93k     1.03k   10.34k    77.99%
  3815863 requests in 10.10s, 189.23MB read
Requests/sec: 377809.30
Transfer/sec:     18.74MB

$ wrk -t 64 -c 64 http://localhost:9999/testolope-nokeepalive
Running 10s test @ http://localhost:9999/testolope-nokeepalive
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   611.05us  225.83us   9.18ms   76.85%
    Req/Sec     1.48k    75.48     2.11k    76.26%
  951970 requests in 10.10s, 64.46MB read
Requests/sec:  94255.61
Transfer/sec:      6.38MB
```
