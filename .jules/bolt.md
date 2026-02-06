## 2025-02-19 - [Topological Sort Performance]
**Learning:** `sorted(deque)` inside a loop turns a linear-ish algorithm into quadratic. Python's Timsort is fast but repeatedly sorting a large queue (width of the graph) is expensive.
**Action:** Use `heapq` for priority queues when order matters, instead of resorting a list/deque. It changes complexity from O(V * W log W) to O(V log W).
