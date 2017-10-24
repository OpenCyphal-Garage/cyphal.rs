# Contributing
Thank you for your interest in contributing to Uavcan.rs! There are many ways to contribute, and we appreciate all of them.

## Questions
If you have a question, feel free to open an issue labeled [QUESTION]. If your question is concerning design choices rather than simple "howto", you can read the [design goals](https://github.com/UAVCAN/uavcan.rs/blob/master/DESIGN.md) first to see if that sheds a bit more light on the problem.

Please have a quick look for duplicates before posting an issue.

## Feature requests
If there is a feature you feel is missing, feel free to open an issue about this. Ideas of how an implementation might be done are appreciated. And if the feature is to be considered application level (rather than a core feature that has not been implemented yet) please elaborate why you think the feature would be useful for the Uavcan community. 

Please have a quick look for duplicates before posting an issue.

## Bug report
If you run into something you think might be a bug, feel free to report it. The better your bug report is, the sooner we will be able to fix it. What you should highly consider including in your bug report.
 - A short summary (I tried to do this and this happened)
 - Code example (preferable minimal that trigger the bug)
 - What surprised you about the code example (When I run this code I expect X but instead Y happens)
 - Meta (rust version, crates.io deps or git deps, uavcan.rs version)

## Pull requests
Pull requests is the way you can help implementing features, fix bugs or improve documentation. We're using the [fork and pull](https://help.github.com/articles/about-collaborative-development-models/) model, where contributors push changes to their personal fork and create pull requests to bring those changes into the source repository.

To save as much time as possible for both yourself and the reviewer, please create an issue discussing the changes before opening a PR. The exception is when you are providing a small patch for a bug fix.

To avoid several working on the same issue, you're encouraged to create the PR early and tag it with [WIP]. If the PR is previously discussed in an issue, please link to it (As previously discussed in #<issue_number>). When you're ready for a review, remove the [WIP] tag and write a message where you tag one of the maintainers (@kjetilkjeka).

Before you start writing code, please have a look at
 - The [design goals](https://github.com/UAVCAN/uavcan.rs/blob/master/DESIGN.md)
 - The Rust [API guidelines](https://rust-lang-nursery.github.io/api-guidelines/about.html)
 - The [Uavcan specification](http://uavcan.org)

Things to remember
 - When changing outwards facing code, update the documentation as well.
 - When adding new features, make sure to add tests to confirm the features is working as intended.
 - If you allow anyone with push access to the upstream repository to make changes to your pull request. Small changes can be applied directly by the reviewer without requiring a new review.
 - Please make sure your code is compatible with our coding style by running `cargo build --feature clippy`.
