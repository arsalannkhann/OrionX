## 2025-02-27 - O(N^2) Topological Sort Anti-Pattern
**Learning:** Found two instances of inefficient topological sort. 1) Using `list.pop(0)` instead of `deque.popleft()` caused O(N^2). 2) Re-sorting the queue every iteration to ensure deterministic order caused O(N^2 log N).
**Action:** Always use `deque` for queues. Use `heapq` when deterministic processing order is required in a graph traversal.
