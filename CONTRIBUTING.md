# Contributing

Thank you for your interest in contributing!

All work on the code base should be motivated by a Github
issue. Before opening a new issue, first do a search of open and closed issues
to make sure that yours will not be a duplicate.
If you would like to work on an issue which already exists, please indicate so
by leaving a comment. If what you'd like to work on hasn't already been covered
by an issue, then open a new one to get the process going.

## Forking

If you do not have write access to the repository, your contribution should be
made through a fork on Github. Fork the repository, contribute to your fork
(either in the `main` branch of the fork or in a separate branch), and then
make a pull request back upstream.

When forking, add your fork's URL as a new git remote in your local copy of the
repo. For instance, to create a fork and work on a branch of it:

- Create the fork on GitHub, using the fork button.
- `cd` to the original clone of the repo on your machine
- `git remote rename origin upstream`
- `git remote add origin git@github.com:<location of fork>`

Now `origin` refers to your fork and `upstream` refers to the original version.
Now `git push -u origin main` to update the fork, and make pull requests
against the original repo.

To pull in updates from the origin repo, run `git fetch upstream` followed by
`git rebase upstream/main` (or whatever branch you're working in).

## Pull Requests

If you have write access to the repo, you can directly branch off of `main`.
This makes it easier for project maintainers to directly make changes to your
branch should the need arise.

Branch names should be prefixed with the author's GitHub username followed by
an associated issue number and short description of the feature, eg.
`username/12-feature-x`.

Pull requests are made against `main` and are squash-merged into main.

PRs must:

- make reference to an issue outlining the context (e.g. `Resolves: #12`)
- update any relevant documentation and include tests

Pull requests should aim to be small and self-contained to facilitate quick
review and merging. Larger change sets should be broken up across multiple PRs.
Commits should be concise but informative, and moderately clean. Commits will be
squashed into a single commit for the PR with all the commit messages.
