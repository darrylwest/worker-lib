# Worker Lib

A framework for supervising an async worker-pool with multiple workers tasks.

```bash
 ________              __                 _____   __ __
|  |  |  |.-----.----.|  |--.-----.----. |     |_|__|  |--.----.---.-.----.--.--.
|  |  |  ||  _  |   _||    <|  -__|   _| |       |  |  _  |   _|  _  |   _|  |  |
|________||_____|__|  |__|__|_____|__|   |_______|__|_____|__| |___._|__| |___  |
                                                                          |_____|
```

## Workers

### Handlers

_Customize a worker for different tasks including database connection, local k/v stores, long running tasks._

### Use Cases

* local in-memory or db backed Key/Value store(s)
* Redis connection workers
* AI Image Processing
* Session storage and activity tracking (example of long-term task, or multi-step session)
* Data Monitoring and Repair
* Sidecar workers in docker K8s, Swarm or other container environment

## Supervisor

The supervisor provides a API to the specific work needed to be done.  

The supervisor also...

* creates and manages a pool of worders (1..)
* dispatches requests by routing to a worker
* monitors worker health, load etc
* recyles old or damaged workers replacing with new

## Implementations

### Cache

* mutople workers
* choice of in-memory or Redis backing
* serialized with JSON storage

### K/V Store

* multple workers
* in-memory only
* typed (no serialization)

## References

* [Wiki Page](https://github.com/darrylwest/worker-lib/wiki)
* Async Std
* Async Channel

###### darryl.west | 2023-01-13

