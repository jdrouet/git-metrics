# Git Metrics

Right now, if you want to track the evolution of some metrics for your project
over time, you need an external tool to store those metrics. But these metrics
could be stored withing the git repository. Git provides a mechanism of notes
that `git-metrics` simplifies.

## How to install

### From sources

```bash
cargo install --git https://github.com/jdrouet/git-metrics
```

## How to use it

```bash
# fetch the remote metrics
git metrics pull
# add a new metric
git metrics add binary-size \
    --tag "platform.arch: amd64" \
    --tag "unit: byte" \
    1234.0
# push the metrics to remote
git metrics push
# log all the metrics for the past commits
git metrics log --filter-empty
# display the metrics on current commit
git metrics show
```

## Project goals

- [x] `git-metrics show` displays the metrics to the current commit
- [x] `git-metrics add` adds a metric to the current commit
- [x] `git-metrics remove` removes a metric from the current commit
- [x] `git-metrics fetch` fetches the metrics
- [x] `git-metrics push` pushes the metrics
- [x] `git-metrics log` displays the metrics for the last commits
- [ ] `git-metrics diff` computes the diff of the metrics between 2 commits
- [ ] `git-metrics page` generates a web page with charts for every metrics