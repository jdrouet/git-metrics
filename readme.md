# Git Metrics

Right now, if you want to track the evolution of some metrics for your project over time, you need an external tool to
store those metrics. But these metrics could be stored withing the git repository. Git provides a mechanism of notes
that `git-metrics` simplifies.

## Project goals

- [x] `git-metrics show` displays the metrics to the current commit
- [x] `git-metrics add` adds a metric to the current commit
- [x] `git-metrics remove` removes a metric from the current commit
- [ ] `git-metrics fetch` fetches the metrics
- [ ] `git-metrics push` pushes the metrics
- [ ] `git-metrics diff` computes the diff of the metrics between 2 commits
- [ ] `git-metrics log` displays the metrics for the last commits
- [ ] `git-metrics page` generates a web page with charts for every metrics
