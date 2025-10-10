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

#### MacBook Pro (`M1 Pro`, `v0.10.0`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   579.95us  474.19us  18.67ms   93.96%
    Req/Sec     1.89k   222.89     3.09k    78.08%
  1213288 requests in 10.11s, 60.17MB read
Requests/sec: 120063.69
Transfer/sec:      5.95MB

$ wrk -t 64 -c 64 http://localhost:9999/testolope-nokeepalive
Running 10s test @ http://127.0.0.1:9999/testolope-nokeepalive
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     3.21ms   13.35ms 163.31ms   97.39%
    Req/Sec   351.54     84.78   500.00     89.42%
  184617 requests in 10.10s, 12.50MB read
  Socket errors: connect 64, read 3, write 0, timeout 0
Requests/sec:  18278.08
Transfer/sec:      1.24MB
```

#### Linux Machine (`Intel(R) Core(TM) i5-10400F CPU @ 2.90GHz`, `v0.10.0`)
```ignore
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   194.38us  138.51us  11.24ms   82.56%
    Req/Sec     5.44k   533.89     9.77k    78.54%
  3496971 requests in 10.10s, 173.42MB read
Requests/sec: 346246.07
Transfer/sec:     17.17MB

$ wrk -t 64 -c 64 http://localhost:9999/testolope-nokeepalive
Running 10s test @ http://localhost:9999/testolope-nokeepalive
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency   677.28us  355.42us  14.77ms   89.79%
    Req/Sec     1.37k   156.80     2.87k    90.07%
  877556 requests in 10.10s, 59.42MB read
Requests/sec:  86883.46
Transfer/sec:      5.88MB
```
