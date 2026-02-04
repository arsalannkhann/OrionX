## 2025-02-14 - Topological Sort Optimization
**Learning:** `deque(sorted(queue))` in a loop creates an O(N^2 log N) bottleneck. Using `heapq` provides the same deterministic order with O(N log N) complexity.
**Action:** When needing a priority queue or deterministic processing order, always prefer `heapq` over sorting a list/deque repeatedly.
