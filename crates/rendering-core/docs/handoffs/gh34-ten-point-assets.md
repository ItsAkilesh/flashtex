# GH34: missing ten-point asset coverage

The existing explicit runtime asset root used by e680d2ef contains ec-lmr12 and
rm-lmr12/8/6, but no ec-lmr10. The default article10pt request requires ec-lmr10.
The recovered capture must remain unchanged; its transport assertions do not
establish zero-diagnostic layout fidelity. This observation does not demonstrate
a rooted-search bug.

A separately labelled replay can add the already acquired official LM2.004 asset:

- Local file: `/tmp/flashtex-lm-tfm-0or97xdj/v2.004/ec-lmr10.tfm`
- Bytes: 12056
- SHA256: `cd13479f463b9a575d053dd7bf0884daa46bfdeffe4b7f537c193861652ac9e5`
- Official archive: `https://www.gust.org.pl/projects/e-foundry/latin-modern/download/lm2.004bas.zip`
- Archive SHA256: `97a725ea012d41367bf44fec1a2f4ccf4fe134c016715522133594e347115a7c`
- Member: `fonts/tfm/public/lm/ec-lmr10.tfm`
- Adjacent `provenance.json` records the original acquisition; adjacent `LICENSE`
  retains the license. No new download, installation, producer change, or substitute
  metrics are required.

Root owns the new runtime capture and must preserve the old capture, identify the
new asset root, record exact resource hashes, and check actual diagnostics. This
handoff only verifies the existing bytes against their recorded provenance; it
makes no fresh producer/search/parity claim.
