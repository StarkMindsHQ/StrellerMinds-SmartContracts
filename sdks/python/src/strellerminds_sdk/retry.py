from __future__ import annotations

import random
import time
from typing import Callable, Optional, TypeVar, Any

T = TypeVar("T")


class RetryOptions:
    def __init__(
        self,
        retries: int = 5,
        initial_delay_ms: int = 200,
        max_delay_ms: int = 10_000,
        multiplier: float = 2.0,
        jitter: bool = True,
        on_retry: Optional[Callable[[BaseException, int, int], None]] = None,
        is_retryable: Optional[Callable[[BaseException], bool]] = None,
    ) -> None:
        self.retries = retries
        self.initial_delay_ms = initial_delay_ms
        self.max_delay_ms = max_delay_ms
        self.multiplier = multiplier
        self.jitter = jitter
        self.on_retry = on_retry
        self.is_retryable = is_retryable


def retry(task: Callable[[], T], options: Optional[RetryOptions] = None) -> T:
    opts = options or RetryOptions()
    attempt = 0
    delay_ms = max(0, int(opts.initial_delay_ms))
    last_error: Optional[BaseException] = None

    while attempt <= opts.retries:
        try:
            return task()
        except BaseException as error:  # noqa: BLE001 - re-raise after policy
            last_error = error
            retryable = opts.is_retryable(error) if opts.is_retryable else True
            if not retryable or attempt == opts.retries:
                raise
            computed_delay = min(
                opts.max_delay_ms,
                int(delay_ms * (1 + random.random())) if opts.jitter else delay_ms,
            )
            if opts.on_retry:
                try:
                    opts.on_retry(error, attempt + 1, computed_delay)
                except Exception:
                    pass
            time.sleep(computed_delay / 1000.0)
            delay_ms = min(
                opts.max_delay_ms, max(1, int(delay_ms * opts.multiplier))
            )
            attempt += 1

    if last_error:
        raise last_error
    raise RuntimeError("Retry failed with unknown error")


def is_transient_network_error(error: BaseException) -> bool:
    message = str(error).lower()
    return any(
        k in message
        for k in [
            "timeout",
            "network",
            "temporary",
            "connection reset",
            "econnreset",
            "econnrefused",
            "503",
            "502",
            "429",
            "rate limit",
            "not_found",
        ]
    )

