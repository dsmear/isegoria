# A2 single-test study, validation (§13.21)

Threshold `t` = 0.7984473698688216 (3fe98ce17fa53de3), frozen in verification/reports/a2-ind-cal-7bbfddb/threshold.tsv; flagged when `I > t`.

## C0 (neutral)

- planned 100; evaluable 99; not evaluable 1 {'candidate not converged': 1}, 95% [0.000, 0.054]
- false alarms: 5 of 99 evaluable, conditional, 95% [0.017, 0.114]; 5 of 100 planned, 95% [0.016, 0.113]
- correct non-flags: 94; zeros of `I`: 30

## C1 (neutral)

- planned 100; evaluable 99; not evaluable 1 {'baseline not converged': 1}, 95% [0.000, 0.054]
- false alarms: 2 of 99 evaluable, conditional, 95% [0.002, 0.071]; 2 of 100 planned, 95% [0.002, 0.070]
- correct non-flags: 97; zeros of `I`: 21

## C3 (unbalanced)

- planned 100; evaluable 96; not evaluable 4 {'baseline not converged': 3, 'candidate not converged': 1}, 95% [0.011, 0.099]
- detections: 23 of 96 evaluable, conditional, 95% [0.158, 0.337]; 23 of 100 planned, 95% [0.152, 0.325]
- misses: 73; zeros of `I`: 18

## C5 (unbalanced)

- planned 100; evaluable 96; not evaluable 4 {'baseline not converged': 4}, 95% [0.011, 0.099]
- detections: 4 of 96 evaluable, conditional, 95% [0.011, 0.103]; 4 of 100 planned, 95% [0.011, 0.099]
- misses: 92; zeros of `I`: 30

## Separation without threshold, conditional on evaluable runs (ties ½)

- C3 against C0: 0.720854377104377 over 96 × 99 pairs
- C3 against C1: 0.7563131313131313 over 96 × 99 pairs
- C5 against C0: 0.47369528619528617 over 96 × 99 pairs
- C5 against C1: 0.5001052188552189 over 96 × 99 pairs

Intervals: Clopper–Pearson, exact two-sided 95%, each marginal; a case's runs taken as independent and identically distributed given the frozen `t`; no adjustment across intervals. Cases are not pooled; nothing is set on validation.
