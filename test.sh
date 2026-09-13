set -euo pipefail
PASSPORT=".github/REPOSITORY_PASSPORT.md"
required=(repo_class visibility ip_sensitivity public_boundary security_profile patent_family pct_deadline canonical_docs release_policy ci_profile runner_profile backup_profile owner)
for f in "${required[@]}"; do
  grep -q "^| ${f} |" "$PASSPORT" || { echo "error $f"; exit 1; }
done
echo OK
