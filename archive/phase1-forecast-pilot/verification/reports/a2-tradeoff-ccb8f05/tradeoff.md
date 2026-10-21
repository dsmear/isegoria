# A2 single-test trade-off (§13.24), post hoc and exploratory

Frozen `t` = 0.7984473698688216 (3fe98ce17fa53de3); flagged when `I > t`. Validation runs used as development material: no threshold is chosen here.

| Case | Planned | Evaluable | Not evaluable | Zeros of `I` |
|---|---|---|---|---|
| C0 | 100 | 99 | {'candidate not converged': 1} | 30 |
| C1 | 100 | 99 | {'baseline not converged': 1} | 21 |
| C3 | 100 | 96 | {'baseline not converged': 3, 'candidate not converged': 1} | 18 |
| C5 | 100 | 96 | {'baseline not converged': 4} | 30 |

## C3/C0: detections of 96 evaluable, false alarms of 99

- frozen `t`: 23/96 detections, 5/99 false alarms; area 0.720854377104377 (the separation, 0.720854377104377)
- 149 points; at `t = 0`, zeros unflagged: 78/96, 69/99

| False alarms at most | Largest detections | At false alarms |
|---|---|---|
| 0/99 | 1/96 | 0/99 |
| 1/99 | 13/96 | 1/99 |
| 2/99 | 13/96 | 1/99 |
| 5/99 | 32/96 | 5/99 |
| 10/99 | 39/96 | 10/99 |
| 20/99 | 62/96 | 20/99 |
| 50/99 | 74/96 | 47/99 |

## C3/C1: detections of 96 evaluable, false alarms of 99

- frozen `t`: 23/96 detections, 2/99 false alarms; area 0.7563131313131313 (the separation, 0.7563131313131313)
- 158 points; at `t = 0`, zeros unflagged: 78/96, 78/99

| False alarms at most | Largest detections | At false alarms |
|---|---|---|
| 0/99 | 14/96 | 0/99 |
| 1/99 | 23/96 | 1/99 |
| 2/99 | 34/96 | 2/99 |
| 5/99 | 43/96 | 5/99 |
| 10/99 | 57/96 | 9/99 |
| 20/99 | 69/96 | 19/99 |
| 50/99 | 75/96 | 46/99 |

## C5/C0: detections of 96 evaluable, false alarms of 99

- frozen `t`: 4/96 detections, 5/99 false alarms; area 0.47369528619528617 (the separation, 0.47369528619528617)
- 137 points; at `t = 0`, zeros unflagged: 66/96, 69/99

| False alarms at most | Largest detections | At false alarms |
|---|---|---|
| 0/99 | 0/96 | 0/99 |
| 1/99 | 1/96 | 1/99 |
| 2/99 | 1/96 | 1/99 |
| 5/99 | 5/96 | 5/99 |
| 10/99 | 12/96 | 10/99 |
| 20/99 | 22/96 | 20/99 |
| 50/99 | 41/96 | 50/99 |

## C5/C1: detections of 96 evaluable, false alarms of 99

- frozen `t`: 4/96 detections, 2/99 false alarms; area 0.5001052188552189 (the separation, 0.5001052188552189)
- 146 points; at `t = 0`, zeros unflagged: 66/96, 78/99

| False alarms at most | Largest detections | At false alarms |
|---|---|---|
| 0/99 | 1/96 | 0/99 |
| 1/99 | 4/96 | 1/99 |
| 2/99 | 7/96 | 2/99 |
| 5/99 | 13/96 | 5/99 |
| 10/99 | 18/96 | 10/99 |
| 20/99 | 26/96 | 19/99 |
| 50/99 | 45/96 | 50/99 |

The pairs behind each area are not independent observations: the runs are, C0 99, C1 99, C3 96, C5 96 evaluable.
