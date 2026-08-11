#!/bin/bash
while true; do
  echo "Checking all branches..."
  PENDING=0
  for branch in bazel-core-infrastructure bazel-rust-icu-sys bazel-other-crates bazel-docs bazel-ci; do
    STATUS=$(gh run list --workflow=bazel.yml --branch $branch -R google/rust_icu --limit 1 --json status | jq -r '.[0].status')
    if [ "$STATUS" != "completed" ]; then
      PENDING=1
      echo "$branch is still $STATUS"
    fi
  done
  if [ "$PENDING" -eq 0 ]; then
    echo "All branches completed!"
    gh run list --workflow=bazel.yml -R google/rust_icu --limit 10
    break
  fi
  sleep 60
done
