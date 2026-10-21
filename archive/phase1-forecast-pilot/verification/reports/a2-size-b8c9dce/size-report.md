# A2 sample-size study (§13.26): C3 against C0 on a single test

- N 3000, C0: planned 16, observed 16, evaluable 15, not evaluable {'baseline not converged': 1}; zeros 2 ['one counted class']; selected {'1 converged': 15, '1 max_iters': 1}; shared candidate {'converged bic_not_lower': 16}
- N 3000, C3: planned 16, observed 16, evaluable 13, not evaluable {'baseline not converged': 3}; zeros 2 ['one counted class']; selected {'1 converged': 13, '1 max_iters': 3}; shared candidate {'converged bic_not_lower': 16}
- N 6000, C0: planned 16, observed 16, evaluable 16, not evaluable 0; zeros 4 ['one counted class']; selected {'1 converged': 16}; shared candidate {'converged bic_not_lower': 16}
- N 6000, C3: planned 16, observed 16, evaluable 15, not evaluable {'baseline not converged': 1}; zeros 1 ['one counted class']; selected {'1 converged': 15, '1 max_iters': 1}; shared candidate {'converged bic_not_lower': 16}

## Separation, every evaluable run

- N 3000: 0.7384615384615385 over 13 × 15 pairs, 4 tied
- N 6000: 0.9416666666666667 over 15 × 16 pairs, 4 tied
- 6,000 minus 3,000: 0.20320512820512815

## Separation, replicates evaluable at both sizes

- N 3000: 0.7166666666666667 over 12 × 15 pairs, 4 tied
- N 6000: 1.0 over 12 × 15 pairs, 0 tied
- 6,000 minus 3,000: 0.2833333333333333, conditional on the common replicates

## Per replicate, evaluable at both sizes

- C0: 15 replicates; I at 6,000 against 3,000 {'greater': 5, 'smaller': 9, 'equal': 1}; zeros {'positive→zero': 3, 'zero→positive': 1, 'positive→positive': 10, 'zero→zero': 1}; evaluable at one size only {9: [6000]}
- C3: 12 replicates; I at 6,000 against 3,000 {'greater': 7, 'smaller': 5, 'equal': 0}; zeros {'positive→positive': 10, 'zero→positive': 2}; evaluable at one size only {0: [6000], 9: [6000], 10: [6000], 14: [3000]}
- C3 placements, reference every evaluable C0 (same size): 12 replicates against 15 and 16 C0; rise 9, fall 0, same 3, not available 0
- C3 placements, reference the common C0 (same size): 12 replicates against 15 and 15 C0; rise 9, fall 0, same 3, not available 0

No interval: the pairs are not independent, the sizes share their samples. No threshold, no success rule; the frozen t is not read here.
