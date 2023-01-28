[![License BSD-2-Clause](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/ehttpd-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/ehttpd-rust)
[![docs.rs](https://docs.rs/ehttpd/badge.svg)](https://docs.rs/ehttpd)
[![crates.io](https://img.shields.io/crates/v/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![Download numbers](https://img.shields.io/crates/d/ehttpd.svg)](https://crates.io/crates/ehttpd)
[![dependency status](https://deps.rs/crate/ehttpd/0.1.0/status.svg)](https://deps.rs/crate/ehttpd/0.1.0)


# `ehttpd`
Welcome to `ehttpd` 🎉

`ehttpd` is a thread-based HTTP server library, which can be used to create custom HTTP server applications.


## Thread-based design
The rationale behind the thread-based approach is that 

## Performance
While the thread-based approach is not the most efficient out there, it's not that bad either. Some `wrk` benchmarks:

### MacBook Pro (`M1 Pro`, keep-alive)
```
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
64 threads and 64 connections
Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.19ms  254.31us   8.62ms   75.59%
    Req/Sec   846.72     55.87     1.29k    72.11%
544736 requests in 10.10s, 27.02MB read
Requests/sec:  53927.49
Transfer/sec:      2.67MB
```

### MacBook Pro (`M1 Pro`, `helloworld`)
```
$ wrk -t 64 -c 64 http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
64 threads and 64 connections
Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.19ms  254.31us   8.62ms   75.59%
    Req/Sec   846.72     55.87     1.29k    72.11%
544736 requests in 10.10s, 27.02MB read
Requests/sec:  53927.49
Transfer/sec:      2.67MB
```

### Old Linux Machine (`Intel(R) Core(TM) i5-2500K CPU @ 3.30GHz`, `helloworld-nokeepalive`)
```
$ wrk -t 64 -c 64 -H "Connection: Close" http://localhost:9999/testolope
Running 10s test @ http://localhost:9999/testolope
  64 threads and 64 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     5.24ms    8.11ms 192.14ms   96.63%
    Req/Sec   226.54     64.93     1.34k    87.69%
  144670 requests in 10.10s, 7.04MB read
  Socket errors: connect 0, read 144670, write 0, timeout 0
Requests/sec:  14326.33
Transfer/sec:    713.64KB
```
