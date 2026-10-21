# A2 single-test study, validation at N = 6,000 (§13.29)

Threshold `t` = 0.641178985817746 (3fe48489cadf0783), frozen in verification/reports/a2-ind-cal6-2a01b81/threshold.tsv; flagged when `I > t`.

## C0 (neutral)

- planned 100; evaluable 100; not evaluable 0 , 95% [0.000, 0.036]
- false alarms: 5 of 100 evaluable, conditional, 95% [0.016, 0.113]; 5 of 100 planned, 95% [0.016, 0.113]
- correct non-flags: 95; zeros of `I`: 29

## C1 (neutral)

- planned 100; evaluable 99; not evaluable 1 {'baseline not converged': 1}, 95% [0.000, 0.054]
- false alarms: 4 of 99 evaluable, conditional, 95% [0.011, 0.100]; 4 of 100 planned, 95% [0.011, 0.099]
- correct non-flags: 95; zeros of `I`: 29

## C3 (unbalanced)

- planned 100; evaluable 100; not evaluable 0 , 95% [0.000, 0.036]
- detections: 46 of 100 evaluable, conditional, 95% [0.360, 0.563]; 46 of 100 planned, 95% [0.360, 0.563]
- misses: 54; zeros of `I`: 18

## C5 (unbalanced)

- planned 100; evaluable 99; not evaluable 1 {'baseline not converged': 1}, 95% [0.000, 0.054]
- detections: 7 of 99 evaluable, conditional, 95% [0.029, 0.140]; 7 of 100 planned, 95% [0.029, 0.139]
- misses: 92; zeros of `I`: 30

## Separation without threshold, conditional on evaluable runs (ties ½)

- C3 against C0: 0.7705 over 100 × 100 pairs
- C3 against C1: 0.7925252525252525 over 100 × 99 pairs
- C5 against C0: 0.5070707070707071 over 99 × 100 pairs
- C5 against C1: 0.5308641975308642 over 99 × 99 pairs

Intervals: Clopper–Pearson, exact two-sided 95%, each marginal; a case's runs taken as independent and identically distributed given the frozen `t`; no adjustment across intervals. Cases are not pooled; nothing is set on validation.
