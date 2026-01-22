## 2024-05-22 - Generational Execution vs Streaming Execution

**Learning:** When executing a DAG of tasks in parallel, a naive "generational" approach (wait for all current tasks to finish before starting next batch) can introduce significant latency, especially if task durations vary widely. The bottleneck is determined by the slowest task in each generation.

**Action:** Use a streaming/waterfall approach where tasks are started immediately as their individual dependencies are met, regardless of other running tasks. In `asyncio`, this can be achieved by maintaining a `running` set and using `asyncio.wait(..., return_when=asyncio.FIRST_COMPLETED)` to process completions as they happen.
