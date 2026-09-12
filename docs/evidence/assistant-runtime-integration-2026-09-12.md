# Assistant and runtime integration

Assistant exactc93bf0d:27 tests including configured original compiler and local
HTTP/subprocess fixtures passed with current ledger/jobs dependencies; strict
all-feature/all-target Clippy passed. No live provider inference. Native UI must
explicitly request provider work and approve the exact reviewed grouped edit.

Runtime exacte5a804d:21 tests including configured original compiler and invalid
UTF8 refusal passed, strict all-target Clippy passed. Event shape and full Value
semantic validation are unchanged. New phase profiles distinguish serialization,
first response byte, frame reading, parsing and source validation.

Paired same-frame parsing benchmark reports median ratio0.89751,19/20 wins; three
scaling runs180/180 exact-clean. Shared-host timing prevents a stable total latency
improvement claim.500KB baseline still spends about251ms before first byte and
about86ms parsing,23ms validation. Compiler owner notified via issues1/21. Native
paint and reference PDF equality remain separate unmet acceptance gates.
