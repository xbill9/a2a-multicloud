# Cross-Cloud Agent Performance Benchmarks

This report summarizes and visualizes the benchmark metrics gathered from calculating Mersenne primes up to exponent 100 across a distributed multi-cloud environment.

## Visual Performance Comparison

````carousel
![Conceptual Infographic Dashboard](/home/xbill/.gemini/antigravity-cli/brain/9887dc96-7a0f-407d-9f82-a680a83d64dd/cloud_benchmark_infographic_1780423098368.png)
<!-- slide -->
![Matplotlib Data-Driven Chart](/home/xbill/.gemini/antigravity-cli/brain/9887dc96-7a0f-407d-9f82-a680a83d64dd/cloud_benchmark_chart.png)
````

---

## Performance Summary Table

| Environment | Tasks Assigned | Cold Start Ready Time (1st task) | Avg Warm Ready Time (Excl. 1st) | Avg Calculation Time |
| :--- | :---: | :---: | :---: | :---: |
| **Rust (Local)** | 25 | 12.23 ms | 21.63 ms | 11.37 ms |
| **AWS (Lightsail)** | 25 | 544.26 ms | 107.94 ms | 110.79 ms |
| **GCP (Cloud Run)** | 25 | 484.59 ms | 127.31 ms | 135.49 ms |
| **Azure (Container Apps)** | 25 | 20,711.57 ms | 348.39 ms | 354.26 ms |

---

## Detailed Analysis

### 1. Ready Time Overhead & Cold Start Latency
* **Azure (Container Apps)** suffered a severe **cold start delay of 20,711.57 ms (~20.7 seconds)** on its first run (exponent 3). Once warmed up, its ready latency dropped to an average of **348.39 ms**, which is still about **3x slower** than both AWS and GCP.
* **AWS** and **GCP** showed robust startup mechanics. Their cold starts were minimal (~500 ms) and their warm ready overheads remained very low (~108 ms and ~127 ms respectively).

### 2. Compute Performance (Computation Time)
* **Local Rust** outperformed the cloud nodes by an order of magnitude (averaging **11.37 ms**), thanks to no networking, container virtualization, or agent orchestration hops.
* **AWS (Lightsail)** led the cloud instances in pure compute performance, completing exponent checks in an average of **110.79 ms**.
* **GCP (Cloud Run)** followed close behind with **135.49 ms**.
* **Azure (Container Apps)** was significantly slower than its cloud peers, averaging **354.26 ms** for calculation (roughly **2.5x slower** than GCP and **3x slower** than AWS).

### 3. prime Exponents Discovered
* **GCP (Cloud Run)**: Discovered **5** primes (Exponents: 5, 13, 17, 61, 89)
* **Azure (Container Apps)**: Discovered **4** primes (Exponents: 3, 7, 19, 31)
* **AWS (Lightsail)**: Discovered **1** prime (Exponent: 2)
* **Rust (Local)**: Discovered **0** primes (only checked even exponents $\ge 4$)
