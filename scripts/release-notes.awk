# Print one version's section of CHANGELOG.md as GitHub release notes.
#
#   awk -v version=0.3.0 -f scripts/release-notes.awk CHANGELOG.md
#
# GitHub keeps every newline in a release body as a line break, so wrapped
# list items (continuation lines indented two spaces) are joined into one line.
# Leading and trailing blank lines are dropped.

# Print a finished line, holding blank lines back until more text follows.
function emit(l) {
    if (l == "") { blanks++; return }
    for (; blanks > 0; blanks--) print ""
    print l
}

BEGIN {
    heading = "## [" version "]"
}

index($0, heading) == 1 { found = 1; next }
found && /^## \[/ { exit }
!found { next }
!have && /^$/ { next }

/^  [^ ]/ && have {
    sub(/^ +/, " ")
    line = line $0
    next
}

{
    if (have) emit(line)
    line = $0
    have = 1
}

END {
    if (have) emit(line)
    if (!found) {
        print "release-notes: no \"" heading "\" section in CHANGELOG.md" > "/dev/stderr"
        exit 1
    }
}
