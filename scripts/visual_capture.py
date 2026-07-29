#!/usr/bin/env python3
"""Capture the real Tartan UI Dioxus gallery in a browser matrix."""

from __future__ import annotations

import json
import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path
from urllib.request import urlopen


TARGET = "tartan-ui/library"
VISUAL_ROOT = Path("target/visual")
VIEWPORTS = (
    ("desktop", 1440, 900),
    ("tablet-landscape", 1024, 768),
    ("tablet-portrait", 768, 1024),
    ("mobile", 390, 844),
)


def free_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def wait_for_http(url: str, process: subprocess.Popen[object]) -> None:
    deadline = time.monotonic() + 180
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"Tartan UI site build exited with status {process.returncode}")
        try:
            with urlopen(url, timeout=1) as response:
                if 200 <= response.status < 400:
                    return
        except OSError:
            pass
        time.sleep(0.5)
    raise TimeoutError(f"timed out waiting for {url}")


def stop_process(process: subprocess.Popen[object]) -> None:
    if process.poll() is not None:
        return
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=5)


def capture_job(revision: str, dirty: bool) -> dict[str, object]:
    return {
        "schema_version": 1,
        "target": TARGET,
        "revision": revision,
        "dirty": dirty,
        "cells": [
            {
                "id": f"gallery/{name}/dark/en-US",
                "path": "/",
                "state": "gallery",
                "viewport": {"width": width, "height": height, "dpr": 1},
                "theme": "dark",
                "locale": "en-US",
                "presets": ["ui-regression"],
            }
            for name, width, height in VIEWPORTS
        ],
    }


def run_capture(root: Path, base_url: str, rubric_root: Path) -> int:
    return subprocess.run(
        [
            "nix",
            "develop",
            "--no-write-lock-file",
            str(rubric_root),
            "-c",
            "cargo",
            "run",
            "--locked",
            "--manifest-path",
            str(rubric_root / "Cargo.toml"),
            "--features",
            "audit",
            "--bin",
            "visual-rubric",
            "--",
            "capture",
            "--root",
            ".",
            "--base-url",
            base_url,
            "--job",
            str(VISUAL_ROOT / "capture_job.json"),
            "--output",
            str(VISUAL_ROOT / "captures"),
            "--manifest",
            str(VISUAL_ROOT / "capture_manifest.json"),
            "--report",
            str(VISUAL_ROOT / "run_report.json"),
            "--browser",
            "chromium",
            "--rubric-workers",
            os.environ.get("VISUAL_RUBRIC_WORKERS", "4"),
            "--cache-dir",
            str(VISUAL_ROOT / "rubric-cache"),
            "--preset",
            "ui-regression",
        ],
        cwd=root,
        check=False,
    ).returncode


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    dirty = bool(subprocess.check_output(["git", "status", "--porcelain=v1"], cwd=root, text=True).strip())
    visual_root = root / VISUAL_ROOT
    visual_root.mkdir(parents=True, exist_ok=True)
    job = capture_job(revision, dirty)
    (visual_root / "capture_job.json").write_text(f"{json.dumps(job, indent=2)}\n")

    rubric_root = Path(os.environ.get("VISUAL_RUBRIC_ROOT", str(root.parent / "visual-rubric"))).resolve()
    if not (rubric_root / "Cargo.toml").is_file():
        raise FileNotFoundError(f"visual-rubric checkout is missing: {rubric_root}")

    build_env = dict(os.environ)
    build_env.pop("RUSTC_WRAPPER", None)
    build_env.pop("SCCACHE_SERVER_UDS", None)
    build_env.pop("NO_COLOR", None)
    output = subprocess.run(
        [
            "dx",
            "build",
            "--web",
            "--release",
            "--locked",
            "--bin",
            "visual_gallery",
            "--features",
            "web",
        ],
        cwd=root / "crates/tartan-ui-dioxus",
        env=build_env,
        capture_output=True,
        text=True,
        check=False,
    )
    if output.returncode != 0:
        raise RuntimeError(f"Tartan UI web fixture build failed:\n{output.stdout}\n{output.stderr}")

    site_root = root / "target/dx/visual_gallery/release/web/public"
    if not site_root.is_dir():
        raise RuntimeError(f"Dioxus web build did not emit its public directory: {site_root}")

    port = free_port()
    server = subprocess.Popen(
        [sys.executable, "-m", "http.server", str(port), "--bind", "127.0.0.1"],
        cwd=site_root,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    try:
        base_url = f"http://127.0.0.1:{port}"
        wait_for_http(f"{base_url}/", server)
        status = run_capture(root, base_url, rubric_root)
    finally:
        stop_process(server)

    report_path = root / VISUAL_ROOT / "run_report.json"
    manifest_path = root / VISUAL_ROOT / "capture_manifest.json"
    if not report_path.is_file() or not manifest_path.is_file():
        raise RuntimeError("visual-rubric did not emit the required report and manifest")
    report = json.loads(report_path.read_text())
    manifest = json.loads(manifest_path.read_text())
    expected = len(job["cells"])
    if report.get("target") != TARGET or manifest.get("target") != TARGET:
        raise RuntimeError("visual-rubric artifact target does not match the producer")
    if report.get("git", {}).get("sha") != revision or manifest.get("revision") != revision:
        raise RuntimeError("visual-rubric artifact revision does not match the producer")
    if report.get("git", {}).get("dirty") is not dirty or manifest.get("dirty") is not dirty:
        raise RuntimeError("visual-rubric artifact dirty state does not match the producer")
    if manifest.get("declared_cells") != expected or report.get("summary", {}).get("total_cells") != expected:
        raise RuntimeError("visual-rubric artifact cell count does not match the producer job")
    return status


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"Tartan UI visual producer failed: {error}", file=sys.stderr)
        sys.exit(1)
