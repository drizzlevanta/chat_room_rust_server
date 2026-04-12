# Work on feature enhancement
When being asked to work on feature enhancement:
- Create a separate git worktree named 'feature/<feature_name>' and work in that separate worktree
- Always use best practices, do not cut corners. If something can't be done, or if you are not sure, create a separate markdown document describing the issue or uncertainty. Do not use workarounds. 
- After you finish, push to remote and create a pull request. 
- Do not create circular dependencies. 

# Importing from external files or crates
- When importing from external files or crates, make sure to do it at the top of the file, not inline.

# Database
- When querying database, minimize the number of the database round trips. 

# Constants
Use `1_000` instead of `1000` for clarity.



