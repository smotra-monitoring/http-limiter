import asyncio
import random
import time
import httpx

URL = "http://localhost:8082/check"
TOTAL_REQUESTS = 200_000
CONCURRENCY = 5

async def send_request(client, semaphore):
    async with semaphore:
        # Generate a random IPv4 address (e.g., to test unique IP limits, 
        # keep the range broad; to test per-IP rate limits, reuse the same few IPs)
        ip = f"10.11.{random.randint(0, 255)}.{random.randint(1, 255)}"
        # ip = f"10.11.12.{random.randint(1, 255)}"
        
        payload = {"ip": ip}
        try:
            start_time = time.perf_counter()
            response = await client.post(URL, json=payload, timeout=5.0)
            latency = (time.perf_counter() - start_time) * 1000 # ms
            return response.status_code, latency
        except Exception as e:
            return 0, 0

async def main():
    semaphore = asyncio.Semaphore(CONCURRENCY)
    limits = httpx.Limits(max_keepalive_connections=CONCURRENCY, max_connections=CONCURRENCY)
    
    print(f"Starting load test: {TOTAL_REQUESTS} requests with concurrency {CONCURRENCY}...")
    
    start_global = time.perf_counter()
    async with httpx.AsyncClient(limits=limits) as client:
        tasks = [send_request(client, semaphore) for _ in range(TOTAL_REQUESTS)]
        results = await asyncio.gather(*tasks)
    
    duration = time.perf_counter() - start_global
    
    # Analyze results
    status_counts = {}
    latencies = []
    for status, latency in results:
        status_counts[status] = status_counts.get(status, 0) + 1
        if latency > 0:
            latencies.append(latency)
        
    print("\n--- Load Test Results ---")
    print(f"Total Time: {duration:.2f} seconds")
    print(f"Requests/sec: {TOTAL_REQUESTS / duration:.2f}")
    print(f"Status Code Breakdown: {status_counts}")
    if latencies:
        latencies.sort()
        print(f"Latency p50: {latencies[int(len(latencies) * 0.50)]:.2f} ms")
        print(f"Latency p99: {latencies[int(len(latencies) * 0.99)]:.2f} ms")

if __name__ == "__main__":
    asyncio.run(main())