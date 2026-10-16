# MXC project management and triage

> **Audience:** MXC developers

The **Microsoft eXecution COntainers** GitHub project is the public planning
board. Anis, Branden, and Gudge own issue and project triage. The
[release and SDK overview](../releases-and-sdk-versioning.md) defines the cadence
and versioning policy.

## Release planning

Use a single-select project field named `Release` with these exact options:

| Release | Meaning |
| --- | --- |
| `VNEXT` | The next planned MXC release, approximately two weeks out |
| `VNEXT +1` | The following release, approximately four weeks out |
| `VFUTURE` | Work not planned for either of those releases |

These are project planning buckets, not repository issue labels. Keep issue
type, area, OS, and `Needs-Triage` labels independent of release planning.
Group a project board view by `Release`, retain the normal `Status` field for
work progress, and make the planned release dates visible in the project.
The dates follow the release windows rather than a strict 14-day timer.

Maintainers place new, unplanned issues in `VFUTURE`; that does not mean the
issue has been reviewed or accepted. Maintainers select work for `VNEXT +1`
and `VNEXT` during triage.

Every two weeks, as a regular release ships:

1. Review and archive the shipped project items, preserving release history.
2. Move remaining `VNEXT +1` work into `VNEXT` and confirm the next target date.
3. Triage `VFUTURE` and select work for the new `VNEXT +1`.
4. Record ownership, scope, and acceptance criteria for selected work.

Keep the two planned windows stable so customers can see approximately a month
of intended work. Carryovers, urgent customer patches, and security needs may
change the plan; maintainers should make those changes explicit rather than
silently reshuffling the board. The board is a plan, not a support guarantee.

## Windows backend bugs

During triage, ensure Windows backend bugs reported on GitHub, including
ProcessContainer/BaseContainer bugs, also have a corresponding work item on
Anis's Tessera Azure DevOps board for management visibility. Anis owns ensuring
that routing is complete. Search for an existing work item before creating one,
include the GitHub issue reference in the internal work item, and keep state
and ownership consistent between the two.

Do not invent an ADO project, area path, or owner when routing details are
missing, or claim routing is complete without verifying the work item. Keep
internal links and sensitive diagnostic material out of public issues; follow
[SECURITY.md](../../SECURITY.md) for security reports and BSODs.
