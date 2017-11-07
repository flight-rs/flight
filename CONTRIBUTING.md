# Contributing to Flight
[contributing-to-flight]: #contributing-to-flight

Flight is already pretty awesome, here is your guide to making it better!

* [Finding an Issue](#issue-triage)
* [Checklist](#checklist)
* [Scripts and Building](#scripts-build)
* [Helpful Links and Information](#helpful-info)

If you have questions, you can talk to Sam (me@samsartor.com).

## Finding an Issue
[issue-triage]: #issue-triage

All issues are carefully labeled according to the following system:
- `A - *`: Area of the codebase
- `P - *`: Priority/schedule
- `Enhancement`:  API/implementation improvement
- `Bug`: Implementation mistake
- `Refactor`: Move/rename API items
- `Rework`: Redesign of a subsystem
- `Spike`: Research/prototyping
- `Good First Issue`: A problem that can be tackled with little flight/Rust experience.

[Good first issue search](https://github.com/CSM-Dream-Team/flight/issues?q=is%3Aopen+is%3Aissue+label%3A%22Good+First+Issue%22+label%3A%22P+-+Next%22)

## Checklist
[checklist]: #checklist

- Document functions/modules/traits/structs/enums/etc. A PR without
  documentation will not be accepted.
- Run `scripts/travis-script.sh`. Is it happy?
- Write some tests. It isn't required, but you still should.
- Submit a PR!

## Scripts and Building
[Scripts and Building]: #scripts-build

We use Travis CI to make sure things don't explode. To check for the same
explosions locally, run `scripts/travis-script.sh`. You can build and view the
docs with `cargo doc --open`. To try an example, `cd` into the relevant
directory (e.g. `examples/intro`) and `cargo run --release`.

## Helpful Links and Information
[helpful-info]: #helpful-info

There will be stuff here someday.
