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

* creates a pool
* allocates workers based on request
* monitors workers
* recyles old or damaged workers


###### darryl.west | 2023-01-09

