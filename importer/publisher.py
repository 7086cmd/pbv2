"""
Zenoh publisher for the PBv2 import pipeline.

The importer publishes ``ImportBatch`` protobuf messages on the key
``pbv2/import/problems``.  The Tauri backend subscribes to this key,
persists each question via the schema crate, and replies with an
``ImportAck`` on ``pbv2/import/ack``.

Key space
─────────
  pbv2/import/problems   importer → backend  (ImportBatch)
  pbv2/import/ack        backend  → importer (ImportAck)

Usage
─────
  from publisher import ImportPublisher
  from pbv2_pb2 import ImportBatch, ImportRequest, ExtractedQuestion

  with ImportPublisher() as pub:
      batch = ImportBatch(source="hw01.pdf", items=[...])
      ack = pub.publish_and_wait(batch)

Environment variables
─────────────────────
  ZENOH_ROUTER   Zenoh router endpoint (default: tcp/127.0.0.1:7447)
  ZENOH_TIMEOUT  Seconds to wait for an ack before raising (default: 30)
"""

from __future__ import annotations

import os
import time
from typing import TYPE_CHECKING

# Proto-generated types (compiled from proto/pbv2.proto via grpcio-tools).
# Run `scripts/compile_proto.py` to regenerate pbv2_pb2.py.
try:
    import pbv2_pb2  # type: ignore[import]
except ModuleNotFoundError:
    pbv2_pb2 = None  # type: ignore[assignment]

if TYPE_CHECKING:
    # Import for type hints only; zenoh may not be installed in all envs.
    import zenoh  # type: ignore[import]

KEY_PROBLEMS = "pbv2/import/problems"
KEY_ACK = "pbv2/import/ack"

_DEFAULT_ROUTER = "tcp/127.0.0.1:7447"
_DEFAULT_TIMEOUT = 30.0


class ImportPublisher:
    """
    Context-manager wrapper around a Zenoh session for publishing import batches.

    Example::

        with ImportPublisher() as pub:
            ack = pub.publish_and_wait(batch)
            if ack.error:
                raise RuntimeError(ack.error)
            print(f"Stored {len(ack.single_problem_ids)} problem(s).")
    """

    def __init__(
        self,
        router: str | None = None,
        timeout: float | None = None,
    ) -> None:
        self._router = router or os.environ.get("ZENOH_ROUTER", _DEFAULT_ROUTER)
        self._timeout = timeout or float(os.environ.get("ZENOH_TIMEOUT", str(_DEFAULT_TIMEOUT)))
        self._session: zenoh.Session | None = None  # type: ignore[name-defined]

    def open(self) -> None:
        import zenoh  # noqa: PLC0415

        config = zenoh.Config()
        config.insert_json5("connect/endpoints", f'["{self._router}"]')
        self._session = zenoh.open(config)

    def close(self) -> None:
        if self._session is not None:
            self._session.close()
            self._session = None

    def __enter__(self) -> "ImportPublisher":
        self.open()
        return self

    def __exit__(self, *_: object) -> None:
        self.close()

    def publish(self, batch: "pbv2_pb2.ImportBatch") -> None:  # type: ignore[name-defined]
        """
        Publish *batch* as a serialised protobuf payload on ``pbv2/import/problems``.

        Fire-and-forget; does not wait for an ack.
        """
        if self._session is None:
            raise RuntimeError("Session not open. Use as a context manager or call open() first.")
        payload = batch.SerializeToString()
        self._session.put(KEY_PROBLEMS, payload)

    def publish_and_wait(
        self,
        batch: "pbv2_pb2.ImportBatch",  # type: ignore[name-defined]
    ) -> "pbv2_pb2.ImportAck":  # type: ignore[name-defined]
        """
        Publish *batch* and block until an ``ImportAck`` is received, or until
        the configured timeout elapses (raises ``TimeoutError``).
        """
        if pbv2_pb2 is None:
            raise RuntimeError(
                "pbv2_pb2 not found. Run scripts/compile_proto.py to generate it."
            )
        if self._session is None:
            raise RuntimeError("Session not open.")

        ack_received: list[pbv2_pb2.ImportAck] = []  # type: ignore[name-defined]

        def _on_sample(sample: "zenoh.Sample") -> None:  # type: ignore[name-defined]
            ack = pbv2_pb2.ImportAck()
            ack.ParseFromString(bytes(sample.payload))
            ack_received.append(ack)

        subscriber = self._session.declare_subscriber(KEY_ACK, _on_sample)
        try:
            self.publish(batch)
            deadline = time.monotonic() + self._timeout
            while not ack_received:
                if time.monotonic() > deadline:
                    raise TimeoutError(
                        f"No ImportAck received within {self._timeout}s "
                        f"(router={self._router!r})"
                    )
                time.sleep(0.05)
        finally:
            subscriber.undeclare()

        return ack_received[0]


# ── Conversion helpers ────────────────────────────────────────────────────────


def question_to_proto(
    q: "schema.ExtractedQuestion",  # type: ignore[name-defined]
) -> "pbv2_pb2.ExtractedQuestion":  # type: ignore[name-defined]
    """Convert a Pydantic :class:`~schema.ExtractedQuestion` to its proto equivalent."""
    if pbv2_pb2 is None:
        raise RuntimeError("pbv2_pb2 not found.")

    def _content(c: "schema.Content") -> "pbv2_pb2.Content":  # type: ignore[name-defined]
        return pbv2_pb2.Content(
            runs=[
                pbv2_pb2.TextRun(
                    text=r.text,
                    is_formula=r.is_formula,
                    bold=r.bold,
                    italic=r.italic,
                )
                for r in c.runs
            ]
        )

    block_kind_map = {
        "none": pbv2_pb2.BLOCK_NONE,
        "essay": pbv2_pb2.BLOCK_ESSAY,
        "proof": pbv2_pb2.BLOCK_PROOF,
        "solve": pbv2_pb2.BLOCK_SOLVE,
    }
    block = pbv2_pb2.QuestionBlock(kind=block_kind_map[q.block_type])
    if q.block_lines is not None:
        block.lines = q.block_lines
    if q.block_space is not None:
        block.space = q.block_space

    proto_q = pbv2_pb2.ExtractedQuestion(
        id=q.id,
        content=_content(q.content),
        sub_questions=[
            pbv2_pb2.SubQuestion(content=_content(sq.content))
            for sq in (q.sub_questions or [])
        ],
        choices=[_content(c) for c in (q.choices or [])],
        block=block,
    )
    if q.answer is not None:
        proto_q.answer.CopyFrom(_content(q.answer))
    if q.solution is not None:
        proto_q.solution.CopyFrom(_content(q.solution))

    return proto_q


def page_to_batch(
    page: "schema.ExtractedPage",  # type: ignore[name-defined]
    category_id: int,
    id_prefix: str,
    source: str,
) -> "pbv2_pb2.ImportBatch":  # type: ignore[name-defined]
    """Convert a full :class:`~schema.ExtractedPage` to an ``ImportBatch`` proto."""
    if pbv2_pb2 is None:
        raise RuntimeError("pbv2_pb2 not found.")

    items = [
        pbv2_pb2.ImportRequest(
            question=question_to_proto(q),
            category_id=category_id,
            id_prefix=id_prefix,
        )
        for q in page.problems
    ]
    return pbv2_pb2.ImportBatch(items=items, source=source)
