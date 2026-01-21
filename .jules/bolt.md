## 2024-05-22 - Asyncio Gather vs Wait
**Learning:** Using `asyncio.gather` for batches of parallel tasks creates unnecessary bottlenecks ("barrier synchronization") when tasks have varying durations. Fast tasks wait for slow tasks before dependent tasks can start.
**Action:** Use `asyncio.wait(return_when=FIRST_COMPLETED)` with a dynamic set of running tasks to implement greedy scheduling, allowing dependent tasks to start as soon as their dependencies complete.
