# Preview performance worker checkpoint

Owner /root, product engineer; sole Commander /root/runtime_validator. Current task FT048revision3 acknowledged after reading remote assignment; scope crates/preview-controller and crates/document-runtime plus own report. Branch agent/commander-preview-performance/preview-performance, worktree /home/natkarri/flashtex-preview-performance. Latest product checkpoint ff5fb39: internal optional delivery queue. Includes runtime150f9f0/controllerf72c44c historical prototypes, c07f7ba full serializer equality evidence and0154d27 actual helper burst evidence. Main now integrates through0154d27; queue awaits integration. Check current Git and newer assignments before continuation.

No active tool jobs, no uncommitted product edits at this checkpoint. Release helper target was rebuilt from4db4a7a for burst evidence; exact artifact hashes are committed. Temporary baseline binaries /tmp/flashtex-helper-legacy-binary (legacy main.rs on same deps) and /tmp/flashtex-helper-owned-binary (7956d37) remain for comparisons. Do not confuse either with latest target. Compiler pin/hash and all limitations are in benchmarks/helper-delivery and helper-burst.

Results: paired10MBwrapping221ms→0.006ms; finalserialization~28ms. Real50KBsequential141ms→95ms (smallsharedLinuxsample). Thirty sequential exact-clean samples/four acknowledged killreopens; added5/50KBlostack scenarios recoveredexactsource, rejectedstaleretry. At30ms typing5KBproduced20previews,50KBonly1; all60durableACKs across3burstcases correct, finalclean and reopenexact. Continuous50KBlivepreview remains unfulfilled. Native painting/referencePDF parity are not measured here.

Next: implement explicitly negotiated helper protocol, authorized by Commander after native agreement issue2 comment5644981151. Default off; negotiate completed-snapshots-v1 and ACK before emitting; reset on helper/compiler restart. Wire historical project/session, original source_versions, compile_revision/current_compile_revision, is_current:false, source_actions_enabled:false, original submission source_binding_token<=128bytes. Drop optional before admission when required output pending, maxone historical/session. main.rs currently has multiproducer required SyncSender (reader failures plus main responses) and writer watchdog2seconds; preserve those semantics while adding scheduling. HistoricalPreview needs consuming result access to avoid cloning; token binding must capture at original compilation, never delayed completion. Read exact native comment before edits. Queue tests5pass; previousfull41+initial4queue tests and finalalltargetlint passed. No active tool handles.

Historical context: Original runtime discarded stale Value before controller received it; do not fake this insidecontroller or weaken is_current_preview. Runtime ownership expansion granted revision3; runtime and internal controller prototypes now implemented and tested; native negotiation/display wiring pending. Proposedsidechannel must be explicitly opt-in, bounded, origin/session-bound, historicalonly, and give no source-action authority. Publish agreement before parallel edits. Compiler throughput remainsindependentrequiredwork.

Staffing: root+font+renderer and soleCommander; drainedbridge/index/ledger remainstopped. No localClaude subscription: API-only withverifiedfundedgrant, noneassumed. Use truthful Codex directGit under authorizedCursor-quota fallback and coauthor authenticatedlocaluser. Root doesnotwrite main/globalcontrol or dispatchremoteagents independently. Allcrossmachineupdates go throughCommander. Do not claimremoteagentsrunning fromqueuesalone.

Compaction: save exactcheckpoint first, useactualtelemetry/nativecontrols; no exposedmanualcompacttool inthisruntime and no guessedpercentages. Never restartCommander or replayuncertainoperations tocompact. Goalcontinuesuntilexplicituserstop.


Latest continuation: negotiated stdio path implemented after b04237d; exact code SHA in next commit. Main.rs integrates output_delivery required FIFO plus single optional frame; completed_protocol token bindings capture generation after synchronous request handling. Protocol example and limits in completed-snapshot-proposal.md. Full52 tests with explicit original compiler and strict lint pass. No outstanding tool jobs. Next publish native handoff through Commander and extend helper_burst.py for opt-in historical counts, original-token checking and lag separate from current latency. Existing strictdefault remains off.


FT048 revision4 now ACKed from main0dc50da, extends ownership to crates/edit-ledger.
Current candidate: Arc immutable history entries, serde rc, nonserialized OnceLock
content-validation cache. No schema or retention change. Ledger66 tests/lint pass;
benchmark raw evidence in edit-ledger/benchmarks/growing-history. Helper integration
first52/53pass + unchanged isolated307page testlaterpass9.56s aftertwo10stimeouts;
actualdebughelper directprobe1.42s. Preserve this qualification. No active tooljobs.
Next improve serialization only with measured semantics-preserving design; receipts,
snapshot imports, backup/undo/redo and I/O uncertainty remain hard gates. Root remains
productengineer; runtime_validator soleCommander. Drainedworkersstaystopped.
