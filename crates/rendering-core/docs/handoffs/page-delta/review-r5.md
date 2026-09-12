# Delta r5: existing-blocker review

Exact proposal 77477b70c34e2250feb89600321888f5b37ff6bb, read-only review.
No delta implementation, producer run, native execution or activation ACK.

The prior variable-field undercharge is addressed at the design level. Section
6.2 replaces fixed per-document/cluster charges with measured header bytes plus
verified per-page lengths. Our 4214-byte document declaration is therefore
charged through its actual header slice; long source paths/text in pages enter
the actual page lengths. Unchanged-page relocation changes only bounded decimal
source offsets, so cached lengths plus their digit changes can be exact for the
specified unchanged producer writer. The previous heuristic is expressly unused.

The prior omitted in-flight preraster set is also addressed in the state model:
installed frame/raster, running input/target/preraster and queued input are all
named, with stale running work retained until terminal. Serialized-byte accounting
is not an RSS bound; the draft explicitly still requires native model overhead
and raster measurements. No measured memory improvement follows from this review.

Remaining implementation acceptance gates, not newly reproduced failures:

1. C3 must verify actual fast-reader raw ranges and writer framing against the
   real producer, including UTF-8, JSON escapes, long variable fields, signed
   relocation and digit boundaries. Python json.dumps is explicitly illustrative.
   Cached raw lengths assume the specified writer spelling; alternate valid JSON
   spellings/whitespace must not silently establish an incorrect exact-size claim.
2. Verify page_bytes has exactly page_count entries, nonnegative bounded values,
   and use checked arithmetic before reconstruction/indexing. Validate relocation
   validity before computing moved decimal widths. The reference is illustrative
   and its early target_bytes call must not replace the full bounded decoder.
3. Zero-page support needs max(N-1,0) separators: the executable reference has
   max(0,N-1), while the prose formula writes N-1 and the schema permits N=0.
4. The one-in-flight invariant must last through completion delivery/publication
   on the main thread, not merely return of the background closure. Exercise a
   delayed main-thread delivery while another request is queued; count every
   retained target/preraster set and preserve stale installation refusal.
5. Run actual producer fresh-full semantic and pixel gates, cap+1 refusal before
   target reconstruction, changed/unchanged page-byte off-by-one refusals, and
   active+queued+painted residency probes. Full resync remains honestly bounded.

No additional demonstrated design blocker was found in the two reviewed areas.
They can move to isolated implementation/testing under owner authorization;
production/helper delta adoption remains pending those gates. Whole-PDF byte
identity is not a fidelity goal. This review concerns display-list accounting
and exact semantics; visual acceptance requires the existing pixel oracle.
