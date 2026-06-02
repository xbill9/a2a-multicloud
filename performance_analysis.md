# Multi-Cloud Agent Performance Analysis (n = 100)

This report details the performance of the distributed agents across GCP, AWS, Azure, and Local (Rust) environments during the Mersenne prime calculation run.

## Performance Chart

![Agent Performance Metrics](/home/xbill/.gemini/antigravity-cli/brain/0ce8e9a9-6bee-434a-a31b-1c94aef7e524/agent_performance.png)

## Latency Summary

| Provider / Environment | Average Ready Check Time | Average Calculation Time | Total Round-Trip Time |
| :--- | :--- | :--- | :--- |
| **Rust (Local)** | **25.2 ms** | **17.9 ms** | **43.1 ms** |
| **AWS (Lightsail)** | **120.3 ms** | **111.9 ms** | **232.2 ms** |
| **Azure (Container Apps)** | **341.2 ms** | **337.8 ms** | **679.0 ms** |
| **GCP (Cloud Run)** | **634.6 ms** | **640.4 ms** | **1,275.0 ms** |

## Key Insights

1. **Local Agent Dominance**: As expected, the local Rust agent (`Rust (Local)`) has the lowest latencies (~43.1 ms total), as it avoids wide-area network routing and TLS handshake overhead.
2. **AWS Lightsail Efficiency**: The AWS Lightsail agent demonstrates impressive network efficiency (~232.2 ms total), significantly outperforming the other remote cloud container offerings.
3. **Azure Container Apps**: Azure ACA exhibits moderate network and execution latencies (~679.0 ms total).
4. **GCP Cloud Run Latency**: GCP Cloud Run demonstrates the highest latencies (~1,275.0 ms total). This is largely due to the extra OIDC identity token generation and bearer token authentication required to query GCP Cloud Run, which adds serialization and network verification steps on every round trip.
