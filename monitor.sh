#!/bin/bash
while true; do
  STATUS=$(gh run list --workflow=bazel.yml --branch bazel-rust-icu-sys -R google/rust_icu --limit 1 --json status,conclusion | jq -r '.[0].status')
  if [ "$STATUS" = "completed" ]; then
    echo "bazel-rust-icu-sys is completed!"
    gh run list --workflow=bazel.yml --branch bazel-rust-icu-sys -R google/rust_icu --limit 1
    break
  fi
  sleep 45
done
