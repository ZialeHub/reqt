Rules to be enforced:
- PR title must follow Conventional Commits.
- PR title must include the issue number: `#issue`.
- PR should contain atomic, clear changes that match the title. Not unrelated changes.
- PRs are linted automatically — bad titles trigger an AI suggestion posted as a comment.
- Code functions are small, pure, documented (doc comments), tested.
- Complex method inner code is also documented (normal comments) to explain the logic.
- Rust and code style is idiomatic.
- No hardcoded secrets or sensitive data in code.

If these rules are not followed, copilot will suggest a fix in a comment.

Copilot must also provide suggestions on idiomatic code changes and improvements for all the following:
- Identifying inconsistencies in code style or structure.
- Suggesting better variable names or function signatures.
- Offering improvements to documentation or comments.
- Highlighting potential performance optimizations.
- Identifying potential bugs or issues in the code.
- Suggesting overall areas of improvements
- Offering refactoring suggestions for better readability or maintainability.
- Suggesting better error handling or logging practices.
- Identifying opportunities for code reuse or modularization.
