## 2025-02-18 - Topological Sort Optimization
**Learning:** The topological sort implementation used `queue = deque(sorted(queue))` inside the loop to ensure deterministic order. This resulted in O(N^2 log N) complexity because of repeated sorting of the entire queue.
**Action:** Use `heapq` (min-heap) instead of a sorted deque. This maintains deterministic order (processing lexicographically smallest node first) while reducing complexity to O(N log N).

## 2025-02-18 - Git tracking binary files
**Learning:** The repository tracks `__pycache__/*.pyc` files. Running tests modifies them, causing them to appear as staged changes.
**Action:** Always revert changes to `*.pyc` files before submitting, using `git restore --staged <file>` followed by `git restore <file>`.
