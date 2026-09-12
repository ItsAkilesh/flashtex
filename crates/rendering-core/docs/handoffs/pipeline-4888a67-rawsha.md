# Reviewable producer raw-digest patch

Base:4888a67 on agent/mac-render-pipeline/unified. Patch owns no authoritative
producer files; it was applied only in /tmp/flashtex-pipeline-rawsha-candidate.
`pipeline-4888a67-rawsha.patch` applies cleanly to an unchanged archived base.

Change only the `typeset.rs` FontResource wire projection. Hash immutable
`LoadedFace::program()` bytes for the resource's `sha256`; retain the existing
engine `font_id`. Do not change LoadedFace.sha256: paragraph-layout's opaque
identity also uses it. Core14 has no program; its existing placeholder and
unsupported-resource behavior are retained. No parser, shaping, spacing, GID,
source map or cache policy changes are included.

On the pinned real LM12/matched TFM fixture, the candidate JSON differs from the
unchanged producer JSON at exactly `/payload/fonts/0/sha256`. Both original IDs
and all geometry/source bytes remain identical. The unmodified downstream
PipelineCff adapter then accepts the candidate and its exact path exporter emits
PDF65d9bc38ea529f163d690f65ba69ce091af3b50a85162e348848fd0f1257fa46.
This is patched-candidate evidence, not a published original build, and no
reference/visual equality is claimed. Machine-readable hashes accompany this
note. The ordinary rendering-core test pins the exact one-leaf change.

Reproduce in a disposable directory from repo root:

```
git archive 4888a67 crates/render-pipeline | tar -x -C /path/to/scratch
git -C /path/to/scratch apply /absolute/path/to/pipeline-4888a67-rawsha.patch
cargo build --offline --manifest-path /path/to/scratch/crates/render-pipeline/Cargo.toml --bin flashtex-render
```

Run that binary with the unchanged request, LM2.004 OTF directory and
FLASHTEX_TFM_DIRS pointing at official LM2.004 metrics, as in the original-reference
README. Pass its actual --v2 file to the existing pipeline_cff_probe example,
with the pinned font/license. Do not rewrite the output JSON. The producer owner
should apply the patch in its own branch, publish the new SHA and rerun its
normal tests. Only then can this consumer rerun published-original acceptance.
