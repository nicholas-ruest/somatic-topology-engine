# Governance fixtures

Durable repository fixtures are generated in isolated temporary directories by
`policy_and_repositories.rs`. Corrupt JSON is deliberately constructed in the
test so it cannot be mistaken for a valid authorization fixture.
