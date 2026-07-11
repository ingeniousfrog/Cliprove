"""In-memory async job tracking for sidecar downloads."""

from __future__ import annotations

import asyncio
import uuid
from dataclasses import dataclass, field
from typing import Any, Awaitable, Callable


@dataclass
class Job:
    job_id: str
    status: str = "queued"
    stage: str = "queued"
    progress: float = 0.0
    error: str | None = None
    result: dict[str, Any] | None = None
    task: asyncio.Task[Any] | None = field(default=None, repr=False)

    def to_dict(self) -> dict[str, Any]:
        return {
            "jobId": self.job_id,
            "status": self.status,
            "stage": self.stage,
            "progress": self.progress,
            "error": self.error,
            "result": self.result,
        }


class JobManager:
    def __init__(self) -> None:
        self._jobs: dict[str, Job] = {}

    def get(self, job_id: str) -> Job | None:
        return self._jobs.get(job_id)

    async def submit(self, coro_factory: Callable[[Job], Awaitable[Any]]) -> Job:
        job = Job(job_id=str(uuid.uuid4()))
        self._jobs[job.job_id] = job

        async def runner() -> None:
            job.status = "running"
            job.stage = "starting"
            job.progress = 0.05
            try:
                job.result = await coro_factory(job)
                job.status = "completed"
                job.stage = "completed"
                job.progress = 1.0
            except Exception as exc:  # noqa: BLE001
                job.status = "failed"
                job.stage = "failed"
                job.error = str(exc)
                job.progress = 0.0

        job.task = asyncio.create_task(runner())
        return job

    def prune(self, max_jobs: int = 200) -> None:
        if len(self._jobs) <= max_jobs:
            return
        completed = [
            job_id
            for job_id, job in self._jobs.items()
            if job.status in {"completed", "failed"}
        ]
        for job_id in completed[: len(self._jobs) - max_jobs]:
            self._jobs.pop(job_id, None)
