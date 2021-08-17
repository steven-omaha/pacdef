# Contributing

Thank you for considering to contribute to `pacdef`. The recommended workflow is this:

1. Open a github issue, mention that you would like to fix the issue in a PR.
2. Wait for approval.
3. Fork the repository and implement your fix / feature.
4. Follow the coding style of this project.

    1. The applicable styles and checks are defined in the github actions. If unsure, ask a developer.
    2. New functions and methods *must* have a cyclomatic complexity of no more than 8. (`$ radon cc -s *.py`).
    3. Changes to existing functions *should* not lead to higher complexity than before.

5. Open the pull request.
