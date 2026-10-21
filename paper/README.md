# Historical working paper

**Opinion as Filter, Evidence as Verdict: The Mathematics of Isegoria, an Authority-Free
Mechanism for Validating Test Items**, version 0.2, September 2026.

This paper analyses the **earlier forecast-and-pilot design**: bridging, IRT/DIF,
forecast-based reviewer reputation and coordination. It does not describe or validate
the offline answer review adopted in D49–D53. For the current product, read the
[overview](../docs/00-overview.md) and [lifecycle](../docs/05-question-lifecycle.md#the-answer-review-01-d49d52).

## Versions and scope

| Material | Scope |
|---|---|
| [Stored PDF](main.pdf) | September snapshot: code `2ee5e79`, decisions D32–D41 at `7507eab`. It lacks the October source amendments. |
| [LaTeX sources](main.tex) | September paper plus the October 11 O1 qualification and October 14 A8/A9 corrections. The historical analysis remains distinct from the current product; the PDF has not been rebuilt. |
| `data/` and `scripts/` | Historical measurements and their generators, not answer-review validation. |

Current finding states and conditional closures belong only in the
[review register](../docs/15-phase1-review.md); task status belongs in the
[roadmap](../docs/10-roadmap.md). Statements of planned work inside the historical paper
refer to its snapshot, not today's backlog. Proofs and measurements retain their stated
hypotheses and domains; none transfers automatically to the answer review.

The directory stays in place because CI's differential oracles import
`scripts/common.py`, and the paper's figures read `data/`. Dependencies are recorded in
the [D47 inventory](../docs/21-d47-inventory.md#4-still-needed-as-dependencies-kept-in-place).

## Layout

| Path | Contents |
|---|---|
| `main.tex`, `preamble.tex`, `sections/` | LaTeX sources |
| `references.bib` | bibliography |
| `data/` | CSV outputs of the scripts; the figures and several tables read them directly |
| `scripts/` | one script per table/figure (see Appendix C of the paper) |
| `scripts/dif-harness/` | runs the production detector (`crates/scoring`) on generated batches |

## Building

```sh
cd paper
latexmk -pdf main.tex        # TeX Live with pgfplots, cleveref, natbib, booktabs
```

## Reproducing the numbers

```sh
cd paper/scripts
pip install -r requirements.txt   # numpy, scipy
python levelA_relativity.py       # Lemma 1 identities, Table 2
python levelA_leak.py             # Table 3, Figure 2 (~8 min)
python levelB_information.py      # Table 4
python levelB_proxy.py            # Table 5 (~2 min)
python levelB_detector.py         # Table 6 and production-detector counts (needs cargo; ~8 min)
python levelB_differential.py     # differential gap (Section 4.5)
python levelC_bss.py              # Figure 3, Table 7
python assignment.py              # Table 8
python revisions_bridging.py      # Tables 10-11 (~8 min)
python revisions_evaluator.py     # Tables 12-13, scenario rates (~5 min)
python revisions_dif.py           # Tables 14-15
python revisions_collusion.py     # Table 16, panel and beacon figures
```

All randomness is seeded. `levelB_detector.py` builds `scripts/dif-harness`, a standalone Cargo
package (its own workspace) that depends on `crates/scoring` by path.

## Disclosure

The analysis, experiments and text were prepared with the assistance of an AI system.
A separate AI-assisted A–E review is tracked in
[`docs/15`](../docs/15-phase1-review.md); its open findings are not an external
certification under T26. Reviews, corrections and objections remain welcome.
