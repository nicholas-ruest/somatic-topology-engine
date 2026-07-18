# Assurance reports

This directory stores reviewed, versioned quality records. Generate reports in
`target/assurance`, inspect them for secrets and sensitive data, then record an
approved copy with:

```bash
bash scripts/record-assurance-report.sh target/assurance/<file> <lowercase-name>.json
```

Every recorded report has a SHA-256 sidecar. A checksum proves integrity, not
authenticity; release evidence is additionally attested by the protected CI
workflow. Never commit raw captures, participant data, credentials, private
keys, or unreviewed machine diagnostics here.
